-- Add raw_message column to store JWE/JWS messages
ALTER TABLE messages ADD COLUMN raw_message TEXT;

-- Add status column to track message acceptance/rejection
ALTER TABLE messages ADD COLUMN status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected'));

-- Create index on status for efficient filtering
CREATE INDEX idx_messages_status ON messages(status);