CREATE TABLE IF NOT EXISTS project_item (
    id SERIAL PRIMARY KEY,
    title VARCHAR UNIQUE NOT NULL,
    thumbnail_img_link VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS project_desc_item (
    id SERIAL PRIMARY KEY,
    project_id INT NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT fk_project_item_to_many_desc FOREIGN KEY (
        project_id
    ) REFERENCES project_item (id)
);

CREATE TABLE IF NOT EXISTS tag (
    id SERIAL PRIMARY KEY,
    text VARCHAR UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS project_tech_tag (
    id SERIAL PRIMARY KEY,
    project_id INT NOT NULL,
    tag_id INT NOT NULL,
    CONSTRAINT fk_project FOREIGN KEY (
        project_id
    ) REFERENCES project_item (id),
    CONSTRAINT fk_tag FOREIGN KEY (
        tag_id
    ) REFERENCES tag (id)
);

CREATE TABLE IF NOT EXISTS tag_category_join (
    id SERIAL PRIMARY KEY,
    tag_id INT NOT NULL,
    tag_category TAG_CATEGORY NOT NULL,
    CONSTRAINT fk_tag FOREIGN KEY (
        tag_id
    ) REFERENCES tag (id)
);

CREATE TYPE tag_category AS ENUM ('language', 'framework', 'database');
