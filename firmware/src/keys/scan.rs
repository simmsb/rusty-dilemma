use core::convert::Infallible;

use embedded_hal_0_2::digital::v2::{InputPin, OutputPin};

pub struct Scanner<C, R>
where
    <R as ScanMatrix<C>>::Debouncers: Default,
    C: ScanColumns,
    R: ScanMatrix<C>,
{
    cols: C,
    rows: R,
    debouncers: <R as ScanMatrix<C>>::Debouncers,
}

impl<C, R> Scanner<C, R>
where
    <R as ScanMatrix<C>>::Debouncers: Default,
    C: ScanColumns,
    R: ScanMatrix<C>,
{
    pub fn new(cols: C, rows: R) -> Self {
        Self {
            cols,
            rows,
            debouncers: Default::default(),
        }
    }

    pub fn scan(&mut self) -> impl Iterator<Item = keyberon::layout::Event> {
        let scan_result = self.rows.scan_matrix(&self.cols, &mut self.debouncers);

        scan_result.into_iter().enumerate().flat_map(|(i, col)| {
            col.into_iter()
                .enumerate()
                .filter_map(move |(j, press_state)| {
                    press_state.map(|press| {
                        if press {
                            keyberon::layout::Event::Press(i as u8, j as u8)
                        } else {
                            keyberon::layout::Event::Release(i as u8, j as u8)
                        }
                    })
                })
        })
    }
}

pub trait ScanColumns {
    type Result: IntoIterator<Item = Option<bool>>;
    type Debouncers;

    fn scan_columns(&self, debouncers: &mut Self::Debouncers) -> Self::Result;
}

const DEBOUNCE_PERIOD: u8 = 4;

impl<C0, C1, C2, C3, C4> ScanColumns for (C0, C1, C2, C3, C4)
where
    C0: InputPin<Error = Infallible>,
    C1: InputPin<Error = Infallible>,
    C2: InputPin<Error = Infallible>,
    C3: InputPin<Error = Infallible>,
    C4: InputPin<Error = Infallible>,
{
    type Result = [Option<bool>; 5];
    type Debouncers = [Debouncer<DEBOUNCE_PERIOD>; 5];

    fn scan_columns(&self, debouncers: &mut Self::Debouncers) -> Self::Result {
        [
            debouncers[0].update(self.0.is_high().unwrap()),
            debouncers[1].update(self.1.is_high().unwrap()),
            debouncers[2].update(self.2.is_high().unwrap()),
            debouncers[3].update(self.3.is_high().unwrap()),
            debouncers[4].update(self.4.is_high().unwrap()),
        ]
    }
}

pub trait ScanMatrix<C: ScanColumns> {
    type Result: IntoIterator<Item = C::Result>;
    type Debouncers;

    fn scan_matrix(&mut self, columns: &C, debouncers: &mut Self::Debouncers) -> Self::Result;
}

impl<C, R0, R1, R2, R3> ScanMatrix<C> for (R0, R1, R2, R3)
where
    C: ScanColumns,
    R0: OutputPin<Error = Infallible>,
    R1: OutputPin<Error = Infallible>,
    R2: OutputPin<Error = Infallible>,
    R3: OutputPin<Error = Infallible>,
{
    type Result = [C::Result; 4];
    type Debouncers = [C::Debouncers; 4];

    fn scan_matrix(&mut self, columns: &C, debouncers: &mut Self::Debouncers) -> Self::Result {
        self.0.set_high().unwrap();
        let a = columns.scan_columns(&mut debouncers[0]);
        self.0.set_low().unwrap();

        self.1.set_high().unwrap();
        let b = columns.scan_columns(&mut debouncers[1]);
        self.1.set_low().unwrap();

        self.2.set_high().unwrap();
        let c = columns.scan_columns(&mut debouncers[2]);
        self.2.set_low().unwrap();

        self.3.set_high().unwrap();
        let d = columns.scan_columns(&mut debouncers[3]);
        self.3.set_low().unwrap();

        [a, b, c, d]
    }
}

pub struct Debouncer<const MAX: u8> {
    integrator: u8,
    is_high: bool,
}

impl<const MAX: u8> Default for Debouncer<MAX> {
    fn default() -> Self {
        Self {
            integrator: Default::default(),
            is_high: Default::default(),
        }
    }
}

impl<const MAX: u8> Debouncer<MAX> {
    const fn new() -> Self {
        Self {
            integrator: 0,
            is_high: false,
        }
    }

    fn is_high(&self) -> bool {
        self.is_high
    }

    fn update(&mut self, is_high: bool) -> Option<bool> {
        if is_high {
            self.increment()
        } else {
            self.decrement()
        }
    }

    fn decrement(&mut self) -> Option<bool> {
        self.integrator = self.integrator.saturating_sub(1);

        if self.integrator == 0 {
            self.is_high = false;
            return Some(false);
        }

        None
    }

    fn increment(&mut self) -> Option<bool> {
        if self.integrator >= MAX {
            self.is_high = true;
            return Some(true);
        } else {
            self.integrator = self.integrator.saturating_add(1);
        }

        None
    }
}
