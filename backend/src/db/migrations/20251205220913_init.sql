create table feeds (
    id varchar(26) primary key not null,
    title text not null,
    feed_url text not null,
    site_url text,
    last_synced_at timestamptz,
    last_sync_result text,
    sync_started_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    unique(feed_url)
);

create table icons (
    id varchar(26) primary key not null,
    hash text not null,
    data bytea not null,
    content_type text not null,
    created_at timestamptz not null default now(),

    unique(hash)
);

create table feeds_icons (
    feed_id varchar(26) not null,
    icon_id varchar(26) not null,
    created_at timestamptz not null default now(),

    primary key (feed_id, icon_id)
);

create table entries (
    id varchar(26) primary key not null,
    feed_id varchar(26) not null,
    title text not null,
    url text not null,
    comments_url text,
    read_at timestamptz,
    starred_at timestamptz,
    published_at timestamptz,
    entry_updated_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    unique(feed_id, url)
);
