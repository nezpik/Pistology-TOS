-- Container table
CREATE TABLE IF NOT EXISTS containers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    container_id TEXT NOT NULL UNIQUE,
    bay TEXT,
    row TEXT,
    tier TEXT,
    size TEXT,
    type TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Location table
CREATE TABLE IF NOT EXISTS locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    container_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_locations_container_id ON locations(container_id);

-- History table
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL,
    description TEXT,
    container_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_history_container_id ON history(container_id);
CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at);
CREATE INDEX IF NOT EXISTS idx_history_event ON history(event);

-- Damage Reports table
CREATE TABLE IF NOT EXISTS damage_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    description TEXT NOT NULL,
    reported_by TEXT NOT NULL,
    photos TEXT, -- JSON array stored as text
    container_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_damage_reports_container_id ON damage_reports(container_id);

-- Customs Inspections table
CREATE TABLE IF NOT EXISTS customs_inspections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status TEXT NOT NULL CHECK(status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED', 'FAILED')),
    notes TEXT,
    inspected_by TEXT,
    container_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_customs_inspections_container_id ON customs_inspections(container_id);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED')),
    assignee TEXT,
    container_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_container_id ON tasks(container_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);

-- Truck Appointments table
CREATE TABLE IF NOT EXISTS truck_appointments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trucking_company TEXT NOT NULL,
    driver_name TEXT NOT NULL,
    license_plate TEXT NOT NULL,
    appointment_time TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('SCHEDULED', 'COMPLETED', 'CANCELLED')),
    container_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_truck_appointments_container_id ON truck_appointments(container_id);
CREATE INDEX IF NOT EXISTS idx_truck_appointments_trucking_company ON truck_appointments(trucking_company);
CREATE INDEX IF NOT EXISTS idx_truck_appointments_status ON truck_appointments(status);
CREATE INDEX IF NOT EXISTS idx_truck_appointments_appointment_time ON truck_appointments(appointment_time);
CREATE INDEX IF NOT EXISTS idx_truck_appointments_license_plate ON truck_appointments(license_plate);

-- EDI Messages table
CREATE TABLE IF NOT EXISTS edi_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_type TEXT NOT NULL,
    content TEXT NOT NULL,
    container_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (container_id) REFERENCES containers(container_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_edi_messages_container_id ON edi_messages(container_id);

-- BAPLIE Messages table
CREATE TABLE IF NOT EXISTS baplie_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    edi_message_id INTEGER NOT NULL UNIQUE,
    vessel_name TEXT,
    voyage_number TEXT,
    port_of_loading TEXT,
    port_of_discharge TEXT,
    FOREIGN KEY (edi_message_id) REFERENCES edi_messages(id) ON DELETE CASCADE
);

-- BAPLIE Containers table
CREATE TABLE IF NOT EXISTS baplie_containers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    baplie_message_id INTEGER NOT NULL,
    container_id TEXT NOT NULL,
    bay TEXT,
    row TEXT,
    tier TEXT,
    size TEXT,
    type TEXT,
    weight REAL,
    FOREIGN KEY (baplie_message_id) REFERENCES baplie_messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_baplie_containers_baplie_message_id ON baplie_containers(baplie_message_id);
CREATE INDEX IF NOT EXISTS idx_baplie_containers_container_id ON baplie_containers(container_id);

-- COARRI Messages table
CREATE TABLE IF NOT EXISTS coarri_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    edi_message_id INTEGER NOT NULL UNIQUE,
    vessel_name TEXT,
    voyage_number TEXT,
    FOREIGN KEY (edi_message_id) REFERENCES edi_messages(id) ON DELETE CASCADE
);

-- COARRI Movements table
CREATE TABLE IF NOT EXISTS coarri_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    coarri_message_id INTEGER NOT NULL,
    container_id TEXT NOT NULL,
    movement_type TEXT CHECK(movement_type IN ('LOAD', 'DISCHARGE', 'SHIFT')),
    stowage_location TEXT,
    iso_container_type TEXT,
    FOREIGN KEY (coarri_message_id) REFERENCES coarri_messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_coarri_movements_coarri_message_id ON coarri_movements(coarri_message_id);
CREATE INDEX IF NOT EXISTS idx_coarri_movements_container_id ON coarri_movements(container_id);
CREATE INDEX IF NOT EXISTS idx_coarri_movements_movement_type ON coarri_movements(movement_type);

-- CODECO Messages table
CREATE TABLE IF NOT EXISTS codeco_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    edi_message_id INTEGER NOT NULL UNIQUE,
    gate TEXT,
    FOREIGN KEY (edi_message_id) REFERENCES edi_messages(id) ON DELETE CASCADE
);

-- CODECO Movements table
CREATE TABLE IF NOT EXISTS codeco_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    codeco_message_id INTEGER NOT NULL,
    container_id TEXT NOT NULL,
    movement_type TEXT CHECK(movement_type IN ('IN', 'OUT')),
    truck_license_plate TEXT,
    iso_container_type TEXT,
    FOREIGN KEY (codeco_message_id) REFERENCES codeco_messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_codeco_movements_codeco_message_id ON codeco_movements(codeco_message_id);
CREATE INDEX IF NOT EXISTS idx_codeco_movements_container_id ON codeco_movements(container_id);
CREATE INDEX IF NOT EXISTS idx_codeco_movements_movement_type ON codeco_movements(movement_type);
CREATE INDEX IF NOT EXISTS idx_codeco_movements_truck_license_plate ON codeco_movements(truck_license_plate);
