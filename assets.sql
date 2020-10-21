create table if not exists assets(
    id          integer primary key autoincrement unique not null,
    created     integer not null default (strftime('%s', 'now')),
    maxage      integer not null,
    key         text unique not null,
    data        blob not null
);
