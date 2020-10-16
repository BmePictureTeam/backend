CREATE extension IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE TABLE app_user(
    id UUID NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin BOOL NOT NULL DEFAULT FALSE
);
CREATE TABLE category(
    id UUID NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    category_name TEXT NOT NULL UNIQUE
);
CREATE TABLE image(
    id UUID NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    upload_date TIMESTAMPTZ,
    title TEXT NOT NULL,
    description TEXT,
    app_user_id UUID NOT NULL REFERENCES app_user(id)
);
CREATE TABLE image_category(
    category_id UUID NOT NULL REFERENCES category(id),
    image_id UUID NOT NULL REFERENCES image(id),
    PRIMARY KEY (category_id, image_id)
);
CREATE TABLE rating(
    app_user_id UUID NOT NULL REFERENCES app_user(id),
    image_id UUID NOT NULL REFERENCES image(id),
    rating INTEGER NOT NULL CHECK(
        RATING BETWEEN 1 AND 5
    ),
    PRIMARY KEY (app_user_id, image_id)
);