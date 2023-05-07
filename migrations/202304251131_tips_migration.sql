create table tips_category
(
    id   integer not null
        constraint tips_category_id
            primary key autoincrement,
    name TEXT not null
);

create table tips
(
    id       integer not null
        constraint id
            primary key autoincrement,
    content  TEXT    not null,
    category INTEGER not null
);
