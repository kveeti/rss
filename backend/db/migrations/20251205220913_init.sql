create table feeds (
    id varchar(26) primary key not null,
    title text not null,
    url text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    unique(url)
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
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    unique(url)
);
