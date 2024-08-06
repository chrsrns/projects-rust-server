CREATE TABLE IF NOT EXISTS blog_item (
    id SERIAL PRIMARY KEY,
    blog_title VARCHAR UNIQUE NOT NULL,
    header_img VARCHAR UNIQUE NOT NULL
);

CREATE TYPE content_type AS ENUM ('bigheader', 'header', 'smallheader', 'body');

CREATE TABLE IF NOT EXISTS content (
    id SERIAL PRIMARY KEY,
    blog_id INT NOT NULL,
    ctype CONTENT_TYPE NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT fk_blog_item FOREIGN KEY (blog_id) REFERENCES blog_item (id)
);
