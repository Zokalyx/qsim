export enum FunctionMode {
    Drawing,
    Formula,   
}
export interface Function {
    name: string,
    mode: FunctionMode,
    formula: string,
    sketching: boolean,
    formula_error: string,
    datapoints: Datapoints | null,
    show_mean: boolean,
    scale: {
        top: number,
        bottom: number,
    },
    visible: boolean,
    complex_phase: number | null,
    readonly: boolean,
    n: number | null,
}
export interface Bounds {
    left: number,
    right: number,
}
// Wrapper required for serde serialization/deserialization
export class Datapoints {
    values: Datapoint[]

    constructor(values: Datapoint[]) {
        this.values = values
    }

    get_path(width: number, height: number, bounds: Bounds, scale: { top: number, bottom: number }): string {
        // Maps selected bounds to screen size
        let left = bounds.left  // -> 0
        let right = bounds.right  // -> WIDTH
        // "Slope" of WIDTH / (right - left)
        // "Root" at left
        // Result: (x - left) * WIDTH / (right - left)

        let bottom = scale.bottom  // -> HEIGHT
        let top = scale.top  // --> 0
        // "Slope" of -HEIGHT / (top - bottom)
        // "Root" at top
        // Result: - (x - top) * HEIGHT / (top - bottom)

        return "M " + this.values.map((datapoint) => {
            let x = (datapoint.x - left) * width / (right - left)
            let y = -(datapoint.y - top) * height / (top - bottom)
            return `${x} ${y}`}).join(" ")
    }

    get_mean(): number {
        const total = this.values
            .map(value => value.y)
            .reduce((partialSum, a) => partialSum + a, 0)
        return this.values
            .map(value => value.x * value.y / total)
            .reduce((partialSum, a) => partialSum + a, 0)
    }
}
export interface Datapoint {
    x: number,
    y: number,
}