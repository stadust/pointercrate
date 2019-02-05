-- This file should undo anything in `up.sql`

DROP TRIGGER demon_addition_trigger ON demons;
DROP FUNCTION audit_demon_addition();
DROP TABLE demon_additions;

DROP TRIGGER demon_modification_trigger ON demons;
DROP FUNCTION audit_demon_modification();
DROP TABLE demon_modifications;

DROP TRIGGER record_addition_trigger ON records;
DROP FUNCTION audit_record_addition();
DROP TABLE record_additions;

DROP TRIGGER record_modification_trigger ON records;
DROP FUNCTION audit_record_modification();
DROP TABLE record_modifications;

DROP TRIGGER record_deletion_trigger ON records;
DROP FUNCTION audit_record_deletion();
DROP TABLE record_deletions;

DROP TRIGGER player_addition_trigger ON players;
DROP FUNCTION audit_player_addition();
DROP TABLE player_additions;

DROP TRIGGER player_modification_trigger ON players;
DROP FUNCTION audit_player_modification();
DROP TABLE player_modifications;

DROP TRIGGER player_deletion_trigger ON players;
DROP FUNCTION audit_player_deletion();
DROP TABLE player_deletions;

-- ...

DROP TABLE audit_log2;

DROP TABLE active_user;