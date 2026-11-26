use std::time::Duration;

use imfield_database::{ports::AssetsDatabaseExt, types::query::Pagination};
use imfield_media::{
    image::{EncodedImageReader, Image, ImageError, ImageFormat, ResizeFilter},
    video::{AudioMode, ExtractVideoFragmentParams, ResizeMode, VideoError},
};
use imfield_mimetype::MimeType;
use imfield_models::{
    DomainError,
    entities::{
        Sidecar,
        asset::{AssetResource, AssetState, AssetStateFailure},
        file::FileVariant,
        sidecar::{SidecarData, SidecarType},
    },
    types::{
        id::{AssetId, FileId},
        media::{Color, ImageHash, ThumbHash},
    },
};
use imfield_storage::{files::ForeignUploader, layout::StorageLayout};
use imfield_storage_address::StoragePath;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::{JobContext, VideoContext, error::JobError};

pub async fn process_asset(
    id: imfield_models::types::id::AssetId,
    ctx: &JobContext,
    cancel: CancellationToken,
) -> Result<(), JobError> {
    // Calls processing for the asset
    _process_asset(id, ctx, &cancel).await
}

pub async fn process_pending_assets_batch(
    ctx: &JobContext,
    cancel: CancellationToken,
) -> Result<(), JobError> {
    let mut handled = 0;

    // Receives unprocessed assets
    let unprocessed = ctx
        .data
        .db
        .assets
        .get_for_processing(Pagination::new(0, 50))
        .await?;

    // Triggers processing for all unprocessed assets
    for asset in unprocessed {
        _process_asset(asset, ctx, &cancel).await?;
        handled += 1;
    }

    Ok(())
}

async fn _process_asset(
    id: AssetId,
    ctx: &JobContext,
    _cancel: &CancellationToken,
) -> Result<(), JobError> {
    // Retrieves an Asset by ID and setting the asset status to `processing`
    let Some(resource) = ctx.data.db.assets.try_start_processing(id).await? else {
        return Ok(());
    };

    // Executes a processing pipeline suitable for the asset type
    let media_type = resource.files.file.mimekind();

    let result = match media_type {
        imfield_mimetype::MimeKind::Image => process_asset_as_img(&resource, &ctx.data).await,
        imfield_mimetype::MimeKind::Video => match &ctx.video {
            Some(vc) => process_asset_as_vid(&resource, &ctx.data, vc).await,
            None => Err(JobError::Video(VideoError::BinUnavailable)),
        },
        imfield_mimetype::MimeKind::Unknown => Err(JobError::Domain(DomainError::Unsupported)),
    };

    // In the event of a processing error, it sets the state to 'failed' and propagates the error
    if let Err(e) = result {
        let failure = match e {
            JobError::Image(ImageError::InvalidDimension)
            | JobError::Image(ImageError::DimensionLimitExceeded) => {
                AssetStateFailure::InvalidDimension
            }
            JobError::Domain(DomainError::Unsupported) => AssetStateFailure::UnsupportedFormat,
            _ => AssetStateFailure::Unknown,
        };

        ctx.data
            .db
            .assets
            .update_state(resource.asset.id(), AssetState::Failed(failure))
            .await?;

        return Err(e);
    }

    Ok(())
}

const THUMBNAIL_WIDTH: u32 = 400;
const THUMBNAIL_QUALITY: usize = 80;

async fn process_asset_as_img(
    resource: &AssetResource,
    ctx: &imfield_di::DataContext,
) -> Result<(), JobError> {
    // Retrieves the original file from the storage
    let path = resource.files.file.path();
    let file = ctx.storage.get_reader(path).await?;

    // Decodes the image from the original file
    let image = Image::from_async_read(file).await?;

    // Checks which variants already exist for the specified asset
    let variants = resource.files.sidecar_variants();

    // Generates and saves a thumbnail
    if !variants.contains(&SidecarType::Thumbnail) {
        let resized = image.resize_by_width(THUMBNAIL_WIDTH, ResizeFilter::Triangle)?;
        let reader = resized.into_reader(ImageFormat::WebP(THUMBNAIL_QUALITY))?;
        store_img_sidecar(
            ctx,
            reader,
            resource.files.file.id(),
            MimeType::Webp,
            SidecarType::Thumbnail,
        )
        .await?;
    }

    // Extract features from the image
    let features = extract_image_features(image)?;

    // Writes features to the database
    ctx.db
        .assets
        .update_fn(resource.asset.id(), move |a| {
            if !matches!(a.state(), AssetState::Processing) {
                return Err(DomainError::BusyState);
            }

            a.set_accent_color(Some(features.color));
            a.set_dimension(Some(features.dim));

            a.set_thumbhash(Some(features.thumbhash));

            let sf = a.similarity_features_mut();
            sf.set_a_hash(features.ahash);
            sf.set_p_hash(features.phash);

            a.set_state(AssetState::Ready);

            Ok(())
        })
        .await?;

    Ok(())
}

const PROBE_TIMEOUT: Duration = Duration::from_mins(2);
const EXTRACT_FRAME_TIMEOUT: Duration = Duration::from_mins(2);
const TRANSCODING_TIMEOUT: Duration = Duration::from_mins(5);

const LOOP_PREVIEW_WIDTH: u32 = 600;
const LOOP_PREVIEW_FRAGMENT_DURATION: Duration = Duration::from_secs(5);

async fn process_asset_as_vid(
    resource: &AssetResource,
    ctx: &imfield_di::DataContext,
    video: &VideoContext,
) -> Result<(), JobError> {
    // Retrieves the original file from the storage
    let path = resource.files.file.path();
    let file = ctx.storage.get_local(path).await?;

    // Get video metadata
    let meta = match timeout(PROBE_TIMEOUT, video.ffprobe.probe_meta(file.path())).await {
        Ok(p) => p?,
        Err(_) => {
            return Err(JobError::ProcessingTimeout);
        }
    };

    // Checks which variants already exist for the specified asset
    let variants = resource.files.sidecar_variants();

    // Generates and saves a loop preview
    if !variants.contains(&SidecarType::LoopPreview) {
        let path = StorageLayout::media_path(
            resource.files.file.id(),
            FileVariant::Sidecar(SidecarType::LoopPreview),
        );

        let uploader = ctx.storage.create_uploader(&path).await?;
        let fragment_duration = LOOP_PREVIEW_FRAGMENT_DURATION.min(meta.video().duration());

        match timeout(
            TRANSCODING_TIMEOUT,
            video.ffmpeg.extract_video_fragment(
                file.path(),
                uploader.path(),
                ExtractVideoFragmentParams {
                    start_at: Duration::from_secs(0),
                    duration: fragment_duration,
                    framerate: None,
                    audio: AudioMode::Disable,
                    resize: ResizeMode::ByWidth(LOOP_PREVIEW_WIDTH),
                },
            ),
        )
        .await
        {
            Ok(r) => r?,
            Err(_) => {
                return Err(JobError::ProcessingTimeout);
            }
        }

        store_vid_sidecar(
            ctx,
            uploader,
            resource.files.file.id(),
            &path,
            MimeType::Mp4,
            SidecarType::LoopPreview,
            fragment_duration,
        )
        .await?;
    }

    let start_frame = {
        let frame = match timeout(
            EXTRACT_FRAME_TIMEOUT,
            video.ffmpeg.read_frame(file.path(), Duration::from_secs(0)),
        )
        .await
        {
            Ok(r) => r?,
            Err(_) => {
                return Err(JobError::ProcessingTimeout);
            }
        };
        let image = Image::from_async_read(frame.as_slice()).await?;
        drop(frame);
        image
    };

    // Generates and saves a thumbnail
    if !variants.contains(&SidecarType::Thumbnail) {
        let resized = start_frame.resize_by_width(THUMBNAIL_WIDTH, ResizeFilter::Triangle)?;
        let reader = resized.into_reader(ImageFormat::WebP(THUMBNAIL_QUALITY))?;
        store_img_sidecar(
            ctx,
            reader,
            resource.files.file.id(),
            MimeType::Webp,
            SidecarType::Thumbnail,
        )
        .await?;
    }

    // Extract features from the image
    let features = extract_image_features(start_frame)?;

    // Writes features to the database
    ctx.db
        .assets
        .update_fn(resource.asset.id(), move |a| {
            if !matches!(a.state(), AssetState::Processing) {
                return Err(DomainError::BusyState);
            }

            a.set_accent_color(Some(features.color));
            a.set_dimension(Some(features.dim));

            a.set_thumbhash(Some(features.thumbhash));

            let sf = a.similarity_features_mut();
            sf.set_a_hash(features.ahash);
            sf.set_p_hash(features.phash);

            a.set_state(AssetState::Ready);

            Ok(())
        })
        .await?;

    Ok(())
}

async fn store_img_sidecar(
    ctx: &imfield_di::DataContext,
    reader: EncodedImageReader,
    id: &FileId,
    mime: MimeType,
    s_ty: SidecarType,
) -> Result<Sidecar, JobError> {
    let path = StorageLayout::media_path(id, FileVariant::Sidecar(s_ty));
    let mut writer = ctx.storage.create_writer(&path).await?;
    writer.write_reader(reader).await?;
    let size = writer.size_bytes();

    let sidecar = Sidecar::new(
        id.clone(),
        s_ty,
        SidecarData {
            path,
            size_bytes: size,
            mimetype: mime,
            duration: None,
        },
    );

    writer.commit().await?;
    ctx.db.files.insert_sidecar(&sidecar).await?;

    Ok(sidecar)
}

async fn store_vid_sidecar(
    ctx: &imfield_di::DataContext,
    uploader: ForeignUploader,
    id: &FileId,
    path: &StoragePath,
    mime: MimeType,
    s_ty: SidecarType,
    duration: Duration,
) -> Result<Sidecar, JobError> {
    let size = uploader.size_bytes().await?;

    let sidecar = Sidecar::new(
        id.clone(),
        s_ty,
        SidecarData {
            path: path.clone(),
            size_bytes: size,
            mimetype: mime,
            duration: Some(duration),
        },
    );

    uploader.finish().await?;
    ctx.db.files.insert_sidecar(&sidecar).await?;

    Ok(sidecar)
}

struct ExtractedImageFeatures {
    dim: (u32, u32),
    color: Color,
    thumbhash: ThumbHash,
    phash: ImageHash,
    ahash: ImageHash,
}

fn extract_image_features(image: Image) -> Result<ExtractedImageFeatures, ImageError> {
    // Get the dimensions
    let (w, h) = (image.width(), image.height());

    // Generates a thumbhash
    let resized = if image.is_landscape() {
        image.resize_by_width(128, ResizeFilter::CatmullRom)
    } else {
        image.resize_by_height(128, ResizeFilter::CatmullRom)
    }?;

    drop(image); // The original is no longer in use; it can be dropped.

    let thumbhash = resized.thumbhash();

    // Calculate the features
    // First, features that require a larger image size are calculated,
    // and then the image is downscaled for each feature
    let p_hash_sized = resized.resize(32, 32, ResizeFilter::Triangle)?;
    let phash = p_hash_sized.p_hash()?;
    drop(resized);

    let a_hash_sized = p_hash_sized.resize(8, 8, ResizeFilter::Triangle)?;
    let ahash = a_hash_sized.a_hash()?;
    drop(p_hash_sized);

    let color = a_hash_sized.average_color();
    drop(a_hash_sized);

    Ok(ExtractedImageFeatures {
        dim: (w, h),
        color,
        thumbhash,
        phash,
        ahash,
    })
}
