CREATE TABLE file_containers (
    id                  VARCHAR(32)     NOT NULL,
    
    original_checksum   BLOB            NOT NULL,
    original_filename   VARCHAR(512),
    original_mimekind   VARCHAR(32)     NOT NULL,

    PRIMARY KEY (id)
);

CREATE INDEX idx_file_containers_checksum ON file_containers(original_checksum);
CREATE INDEX idx_file_containers_mimekind ON file_containers(original_mimekind);

CREATE TABLE files (
    container_id        VARCHAR(32)     NOT NULL,
    variant             VARCHAR(32)     NOT NULL,

    created_at          TIMESTAMPTZ     NOT NULL,

    path                VARCHAR(512)    NOT NULL,

    size_bytes          BIGINT          NOT NULL,
    mimetype            VARCHAR(128)    NOT NULL,

    duration_ms         BIGINT,

    PRIMARY KEY (container_id, variant),

    FOREIGN KEY (container_id) REFERENCES file_containers(id) ON DELETE CASCADE,

    CONSTRAINT  valid_duration  CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CONSTRAINT  valid_size      CHECK (size_bytes >= 0)
);

CREATE INDEX idx_file_container ON files(container_id);

CREATE TABLE assets (
    id                  BIGINT          NOT NULL,
    file_container_id   VARCHAR(32)     NOT NULL,

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

    FOREIGN KEY (file_container_id) REFERENCES file_containers(id),

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
