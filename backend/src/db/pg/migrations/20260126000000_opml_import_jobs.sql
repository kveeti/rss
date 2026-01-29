create table opml_import_jobs (
    id varchar(26) primary key not null,
    status text not null,
    total bigint not null,
    imported bigint not null default 0,
    skipped bigint not null default 0,
    failed bigint not null default 0,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table opml_import_items (
    id varchar(26) primary key not null,
    job_id varchar(26) not null references opml_import_jobs(id) on delete cascade,
    feed_url text not null,
    status text not null,
    error text,
    created_at timestamptz not null default now(),
    updated_at timestamptz,
    unique (job_id, feed_url)
);

create index opml_import_items_job_id_idx on opml_import_items(job_id);
