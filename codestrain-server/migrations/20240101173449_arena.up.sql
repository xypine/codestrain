-- Add up migration script here
CREATE TABLE IF NOT EXISTS battles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    arena_size INT NOT NULL,
    strain_a UUID NOT NULL REFERENCES strains(id) ON DELETE CASCADE,
    strain_b UUID NOT NULL REFERENCES strains(id) ON DELETE CASCADE,
    score_a INT NOT NULL,
    score_b INT NOT NULL,
    winner UUID REFERENCES strains(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (strain_a, strain_b, arena_size)
);

CREATE TABLE IF NOT EXISTS battle_logs (
    battle_id UUID NOT NULL REFERENCES battles(id) ON DELETE CASCADE,
    turn INT NOT NULL,
    move_x INT NOT NULL,
    move_y INT NOT NULL,
    last boolean NOT NULL,
    allowed boolean NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (battle_id, turn)
);