CREATE TABLE `attendance` (
    `student` TEXT NOT NULL,
    `week` INTEGER NOT NULL,
    `status` TEXT NOT NULL,
    FOREIGN KEY (`student`) REFERENCES students (id) ON DELETE CASCADE,
    FOREIGN KEY (`week`) REFERENCES weeks (id) ON DELETE CASCADE,
    PRIMARY KEY (`student`, `week`)
);
