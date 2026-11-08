CREATE TABLE files (
    id                  VARCHAR(32)     NOT NULL,

    created_at          TIMESTAMPTZ     NOT NULL,

    path                VARCHAR(512)    NOT NULL,
    size_bytes          BIGINT          NOT NULL,

    mimetype            VARCHAR(128)    NOT NULL,
    mimekind            VARCHAR(32)     NOT NULL,

    duration_ms         BIGINT,

    original_filename   VARCHAR(512),
    checksum            BLOB            NOT NULL,

    PRIMARY KEY (id),

    CONSTRAINT  valid_duration  CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CONSTRAINT  valid_size      CHECK (size_bytes >= 0)
);

CREATE INDEX idx_files_checksum ON files(checksum);
CREATE INDEX idx_files_mimekind ON files(mimekind);

CREATE TABLE sidecars (
    file_id             VARCHAR(32)     NOT NULL,
    sidecar_type        VARCHAR(64)     NOT NULL,

    created_at          TIMESTAMPTZ     NOT NULL,

    path                VARCHAR(512)    NOT NULL,
    size_bytes          BIGINT          NOT NULL,

    mimetype            VARCHAR(128)    NOT NULL,
    duration_ms         BIGINT,

    PRIMARY KEY (file_id, sidecar_type),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,

    CONSTRAINT  valid_sidecar_duration  CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CONSTRAINT  valid_sidecar_size      CHECK (size_bytes >= 0)
);

CREATE TABLE assets (
    id                  BIGINT          NOT NULL,
    file_id             VARCHAR(32)     NOT NULL,

    accent_color        INTEGER,
    width               INTEGER,
    height              INTEGER,

    thumbhash           BLOB,

    title               VARCHAR(255),
    caption             VARCHAR(512),
    source_url          VARCHAR(512),

    state               VARCHAR(32)     NOT NULL,
    state_failure       VARCHAR(128),

    is_offline          BOOLEAN         NOT NULL,

    parent_id           BIGINT,
    duplicate_of        BIGINT,

    created_at          TIMESTAMPTZ     NOT NULL,
    updated_at          TIMESTAMPTZ,
    deleted_at          TIMESTAMPTZ,

    PRIMARY KEY (id),

    FOREIGN KEY (file_id)       REFERENCES files(id),

    CONSTRAINT  valid_parent    CHECK (id <> parent_id),
    FOREIGN KEY (parent_id)     REFERENCES assets(id)  ON DELETE SET NULL,

    CONSTRAINT  valid_duplicate CHECK (id <> duplicate_of),
    FOREIGN KEY (duplicate_of)  REFERENCES assets(id)  ON DELETE SET NULL,

    CONSTRAINT  valid_dimension CHECK (
        (
            (width IS NULL) AND (height IS NULL)
        )
        OR (
            (width IS NOT NULL AND width > 0) AND (height IS NOT NULL AND height > 0)
        )
    )
);

CREATE INDEX idx_assets_state           ON assets(state);

CREATE INDEX idx_assets_parent          ON assets(parent_id);
CREATE INDEX idx_assets_duplicate       ON assets(duplicate_of);

CREATE INDEX idx_assets_file_container  ON assets(file_container_id);

CREATE TABLE assets_features (
    id                  BIGINT          NOT NULL,

    updated_at          TIMESTAMPTZ,

    p_hash              BIGINT,
    a_hash              BIGINT,

    PRIMARY KEY (id),

    FOREIGN KEY (id) REFERENCES assets(id) ON DELETE CASCADE
);
