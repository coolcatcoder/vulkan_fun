pub const VERTEX_TABLE: [&[(i8, i8)]; 16] = [
    &[],                                                   // confirmed
    &[(-1, 0), (0, -1), (-1, -1)],                         // confirmed
    &[(1, 0), (1, -1), (0, -1)],                           // confirmed
    &[(-1, 0), (-1, -1), (1, 0), (1, -1)],                 // confirmed
    &[(0, 1), (1, 1), (1, 0)],                             // confirmed
    &[(-1, 0), (0, -1), (-1, -1), (0, 1), (1, 1), (1, 0)], // confirmed abiguous
    &[(0, 1), (0, -1), (1, 1), (1, -1)],                   // confirmed
    &[(-1, 0), (-1, -1), (0, 1), (1, 1), (1, -1)],         // confirmed
    &[(-1, 1), (0, 1), (-1, 0)],                           // confirmed
    &[(-1, 1), (-1, -1), (0, 1), (0, -1)],                 // confirmed
    &[(-1, 1), (0, 1), (-1, 0), (1, 0), (1, -1), (0, -1)], // confirmed ambiguous
    &[(-1, 1), (-1, -1), (0, 1), (1, 0), (1, -1)],         // confirmed
    &[(-1, 1), (-1, 0), (1, 1), (1, 0)],                   // confirmed
    &[(-1, 1), (-1, -1), (0, -1), (1, 1), (1, 0)],         // confirmed
    &[(-1, 1), (-1, 0), (0, -1), (1, 1), (1, -1)],         // confirmed
    &[(-1, 1), (-1, -1), (1, 1), (1, -1)],                 // confirmed
];

pub const INDEX_TABLE: [&[u32]; 16] = [
    &[],
    &[0, 1, 2],
    &[0, 1, 2],
    &[0, 2, 1, 1, 2, 3],
    &[0, 1, 2],
    &[0, 1, 2, 3, 4, 5],
    &[0, 2, 3, 0, 3, 1],
    &[0, 4, 1, 0, 2, 4, 2, 3, 4],
    &[0, 1, 2],
    &[0, 2, 3, 0, 3, 1],
    &[0, 1, 2, 3, 4, 5],
    &[0, 2, 1, 2, 3, 1, 3, 4, 1],
    &[0, 3, 1, 0, 2, 3],
    &[0, 2, 1, 0, 4, 2, 0, 3, 4],
    &[0, 3, 1, 1, 3, 2, 2, 3, 4],
    &[0, 2, 1, 2, 3, 1],
];

pub const fn get_square_index(corners: [bool; 4]) -> u8 {
    let mut square_index = 0u8;

    if corners[0] {
        square_index |= 1;
    }
    if corners[1] {
        square_index |= 2;
    }
    if corners[2] {
        square_index |= 4;
    }
    if corners[3] {
        square_index |= 8;
    }

    square_index
}
