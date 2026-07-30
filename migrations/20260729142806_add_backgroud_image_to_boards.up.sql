ALTER TABLE boards ADD COLUMN background_image_attachment_id uuid NULL,
ADD CONSTRAINT fkey_boards_to_background_image_attachments
    FOREIGN KEY (background_image_attachment_id) REFERENCES attachments(id) ON DELETE SET NULL;
