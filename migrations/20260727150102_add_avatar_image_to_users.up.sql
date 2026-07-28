ALTER TABLE users ADD COLUMN avatar_image_attachment_id uuid NULL,
ADD CONSTRAINT fkey_users_to_avatar_image_attachments
    FOREIGN KEY (avatar_image_attachment_id) REFERENCES attachments(id) ON DELETE SET NULL;
