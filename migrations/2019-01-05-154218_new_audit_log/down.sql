-- This file should undo anything in `up.sql`

DROP TABLE demon_additions;
DROP FUNCTION audit_demon_addition() CASCADE;

DROP TABLE demon_modifications;
DROP FUNCTION audit_demon_modification() CASCADE;

DROP TABLE record_additions;
DROP FUNCTION audit_record_addition() CASCADE;

DROP TABLE record_modifications;
DROP FUNCTION audit_record_modification() CASCADE;

DROP TABLE record_deletions;
DROP FUNCTION audit_record_deletion() CASCADE;

DROP TABLE player_additions;
DROP FUNCTION audit_player_addition() CASCADE;

DROP TABLE player_modifications;
DROP FUNCTION audit_player_modification() CASCADE;

DROP TABLE player_deletions;
DROP FUNCTION audit_player_deletion() CASCADE;

DROP TABLE creator_additions;
DROP FUNCTION audit_creator_addition() CASCADE;

DROP TABLE creator_deletions;
DROP FUNCTION audit_creator_deletion() CASCADE;

DROP TABLE submitter_modifications;
DROP FUNCTION audit_submitter_modification() CASCADE;

DROP TABLE user_additions;
DROP FUNCTION audit_user_addition() CASCADE;

DROP TABLE user_modifications;
DROP FUNCTION audit_user_modification() CASCADE;

DROP TABLE user_deletions;
DROP FUNCTION audit_user_deletion() CASCADE;

DROP TABLE audit_log2;

DROP TABLE active_user;