// PID: Proportional-Integral-Derivative control (needs testing TODO)
use std::collections::VecDeque;

// PID controller object. `T` should be a numeric type.
struct PIDController<T> {
    // > The P term is the error of `current` to the passed argument with a
    // > ... gain term
    proportion: T,
    // > The I term adds previous delta `current` values
    integral: VecDeque<T>,
    // > The I term should have a fixed but adjustable size
    integral_size: usize,
    // > The D term is the current rate of change in `current`
    derivative: T,
    // > The previous D term is needed to calculate a derivative
    derivative_previous: T,

    current: T,

    // > Weights for the PID controller
    weight_proportion: T,
    weight_integral: T,
    weight_derivative: T,

    // > Minimum value this PID controller can hit
    min: T,
    // > Maximum value this PID controller can hit
    max: T,
}

impl PIDController<f32> {
    fn new(
        min: f32,
        max: f32,
        weight_proportion: f32,
        weight_integral: f32,
        weight_derivative: f32,
        integral_size: usize,
        start_at: f32,
        start_proportion: f32,
        start_derivative: f32,
    ) -> PIDController<f32> {
        PIDController {
            proportion: start_proportion,
            integral: vec![0.0_f32; integral_size].into_iter().collect(),
            integral_size: integral_size,
            derivative: start_derivative,
            derivative_previous: start_derivative,
            current: start_at,
            weight_proportion: weight_proportion,
            weight_integral: weight_integral,
            weight_derivative: weight_derivative,
            min: min,
            max: max,
        }
    }

    fn adjust(&mut self, value: f32) -> f32 {
        // adjust proportion here
        self.proportion = value - self.current;

        // adjust integral here
        while self.integral_size < self.integral.len() {
            // the integral is too large, pop desired elems
            self.integral.pop_back();
        }
        while (self.integral_size - 1) > self.integral.len() {
            // the integral is too small, repeatedly zero
            self.integral.push_front(0.0_f32);
        }
        // finally the integral is good, push in a zero
        self.integral.push_front(self.proportion);

        // adjust derivative here
        let derivative_hold = self.derivative_previous;
        self.derivative_previous = self.derivative;
        self.derivative = self.proportion - derivative_hold;

        // calc result
        let result: f32 = (self.proportion * self.weight_proportion)
            + (self.integral.clone().into_iter().sum::<f32>() / (self.integral.len() as f32))
            + (self.derivative * self.weight_derivative);
        assert_eq!(
            result.is_nan(),
            false,
            "PID control resulted in abnormal NaN delta"
        );

        // set current then clamp
        self.current = self.current + result;
        assert_eq!(
            self.current.is_nan(),
            false,
            "PID control resulted in abnormal NaN current value"
        );
        self.current = self.current.clamp(self.min, self.max);
        
        self.current
    }
}
