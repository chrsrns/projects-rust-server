CREATE TABLE IF NOT EXISTS project_item (
    id SERIAL PRIMARY KEY,
    title VARCHAR UNIQUE NOT NULL,
    thubnail_img_link VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS project_desc_item (
    id SERIAL PRIMARY KEY,
    project_id INT NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT fk_project_item_to_many_desc FOREIGN KEY (
        project_id
    ) REFERENCES project_item (id)
);
