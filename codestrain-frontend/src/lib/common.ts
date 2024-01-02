export type StrainInput = {
    board: [[number, number], boolean | null][],
    allowed: [number, number][]
};

export type StrainOutput = [number, number];