//! One sun and moon model for the whole game: the situation map terminator, the
//! tactical sky state and weather choice all read from here, so a site's light
//! on the globe always agrees with its light on the battlefield.
//!
//! Inputs are the game clock in UTC. Accuracy target: sun within about one
//! degree, which is far finer than anything the picture can show.

use std::f32::consts::TAU;

/// Days in each month of a non-leap year.
const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Day of year, 1..=366. `month` is 1..=12, `day` is 1..=31.
pub fn day_of_year(year: i32, month: i32, day: i32) -> i32 {
    let mut n = day;
    for m in 0..(month - 1).clamp(0, 11) as usize {
        n += MONTH_DAYS[m];
        if m == 1 && is_leap(year) {
            n += 1;
        }
    }
    n
}

/// Whole days since 2000-01-01 (used for the moon's phase).
pub fn days_since_2000(year: i32, month: i32, day: i32) -> i32 {
    let mut n = 0;
    if year >= 2000 {
        for y in 2000..year {
            n += if is_leap(y) { 366 } else { 365 };
        }
    } else {
        for y in year..2000 {
            n -= if is_leap(y) { 366 } else { 365 };
        }
    }
    n + day_of_year(year, month, day) - 1
}

/// A moment on the game clock, UTC.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moment {
    pub year: i32,
    /// 1..=12
    pub month: i32,
    /// 1..=31
    pub day: i32,
    /// Minutes since midnight UTC; fractional so turns can advance it smoothly.
    pub minute: f32,
}

impl Moment {
    pub fn new(year: i32, month: i32, day: i32, minute: f32) -> Self {
        Self { year, month, day, minute }
    }

    /// The same moment `secs` seconds later (day rollover handled; good for
    /// the length of a battle, not for calendar arithmetic across months).
    pub fn plus_seconds(self, secs: f32) -> Self {
        let mut m = self;
        m.minute += secs / 60.0;
        while m.minute >= 1440.0 {
            m.minute -= 1440.0;
            m.day += 1;
            let mut len = MONTH_DAYS[(m.month - 1).clamp(0, 11) as usize];
            if m.month == 2 && is_leap(m.year) {
                len += 1;
            }
            if m.day > len {
                m.day = 1;
                m.month += 1;
                if m.month > 12 {
                    m.month = 1;
                    m.year += 1;
                }
            }
        }
        m
    }

    fn fractional_year(self) -> f32 {
        let n = day_of_year(self.year, self.month, self.day) as f32;
        TAU / 365.0 * (n - 1.0 + (self.minute / 60.0 - 12.0) / 24.0)
    }

    /// Sun's declination in radians (+ north). Swings ±23.44° over the year.
    pub fn declination(self) -> f32 {
        let g = self.fractional_year();
        0.006918 - 0.399912 * g.cos() + 0.070257 * g.sin() - 0.006758 * (2.0 * g).cos()
            + 0.000907 * (2.0 * g).sin()
            - 0.002697 * (3.0 * g).cos()
            + 0.00148 * (3.0 * g).sin()
    }

    /// Equation of time in minutes (sundial minus clock).
    pub fn equation_of_time(self) -> f32 {
        let g = self.fractional_year();
        229.18
            * (0.000075 + 0.001868 * g.cos() - 0.032077 * g.sin() - 0.014615 * (2.0 * g).cos()
                - 0.040849 * (2.0 * g).sin())
    }

    /// Point on Earth where the sun is overhead: (latitude, longitude) degrees.
    pub fn subsolar(self) -> (f32, f32) {
        let lat = self.declination().to_degrees();
        let lon = -(self.minute + self.equation_of_time() - 720.0) / 4.0;
        (lat, wrap_deg(lon))
    }

    /// Local solar time at a longitude, minutes since local midnight.
    pub fn solar_minute(self, lon_deg: f32) -> f32 {
        (self.minute + self.equation_of_time() + 4.0 * lon_deg).rem_euclid(1440.0)
    }

    /// Sun as seen from a site.
    pub fn sun_at(self, lat_deg: f32, lon_deg: f32) -> SkyPos {
        let (slat, slon) = self.subsolar();
        local_pos(lat_deg, lon_deg, slat, slon)
    }

    /// Moon phase 0..1: 0 new, 0.5 full.
    pub fn moon_phase(self) -> f32 {
        // Reference new moon: 2000-01-06 18:14 UTC. Synodic month 29.530588 d.
        let d = days_since_2000(self.year, self.month, self.day) as f32 + self.minute / 1440.0;
        ((d - 5.76) / 29.530588).rem_euclid(1.0)
    }

    /// Fraction of the moon's disc that is lit, 0..1.
    pub fn moon_illumination(self) -> f32 {
        0.5 * (1.0 - (self.moon_phase() * TAU).cos())
    }

    /// Moon as seen from a site. A simple model: the moon trails the sun by
    /// its phase angle along a path tilted like the sun's six months away at
    /// full moon. Good enough to place moonlight and its shadows believably.
    pub fn moon_at(self, lat_deg: f32, lon_deg: f32) -> SkyPos {
        let (slat, slon) = self.subsolar();
        let ph = self.moon_phase();
        let mlon = wrap_deg(slon - ph * 360.0);
        let mlat = slat * (ph * TAU).cos() + 5.1 * (ph * TAU * 13.0 / 12.0).sin();
        local_pos(lat_deg, lon_deg, mlat, mlon)
    }
}

/// Position of a body in a site's sky.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyPos {
    /// Radians above the horizon (negative when below).
    pub elevation: f32,
    /// Radians clockwise from north (east = π/2).
    pub azimuth: f32,
}

impl SkyPos {
    /// Unit vector toward the body in map space: +x east, +y up, +z south.
    pub fn dir(self) -> [f32; 3] {
        let ce = self.elevation.cos();
        [ce * self.azimuth.sin(), self.elevation.sin(), -ce * self.azimuth.cos()]
    }
}

fn wrap_deg(d: f32) -> f32 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

fn local_pos(lat_deg: f32, lon_deg: f32, body_lat_deg: f32, body_lon_deg: f32) -> SkyPos {
    let lat = lat_deg.to_radians();
    let dec = body_lat_deg.to_radians();
    let h = wrap_deg(lon_deg - body_lon_deg).to_radians(); // hour angle, + west of the meridian
    let sin_el = lat.sin() * dec.sin() + lat.cos() * dec.cos() * h.cos();
    let elevation = sin_el.clamp(-1.0, 1.0).asin();
    // Azimuth from north, clockwise.
    let y = -h.sin() * dec.cos();
    let x = dec.sin() * lat.cos() - dec.cos() * lat.sin() * h.cos();
    let azimuth = y.atan2(x).rem_euclid(TAU);
    SkyPos { elevation, azimuth }
}

/// The nine baked sky states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyState {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    GoldenHour,
    Dusk,
    Night,
    Overcast,
    Storm,
}

impl SkyState {
    pub const ALL: [SkyState; 9] = [
        SkyState::Dawn,
        SkyState::Morning,
        SkyState::Noon,
        SkyState::Afternoon,
        SkyState::GoldenHour,
        SkyState::Dusk,
        SkyState::Night,
        SkyState::Overcast,
        SkyState::Storm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SkyState::Dawn => "dawn",
            SkyState::Morning => "morning",
            SkyState::Noon => "noon",
            SkyState::Afternoon => "afternoon",
            SkyState::GoldenHour => "golden_hour",
            SkyState::Dusk => "dusk",
            SkyState::Night => "night",
            SkyState::Overcast => "overcast",
            SkyState::Storm => "storm",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn from_name(s: &str) -> Option<SkyState> {
        let k = s.to_ascii_lowercase().replace(['-', ' '], "_");
        Self::ALL.iter().copied().find(|st| st.name() == k)
    }
}

/// Which two clear-sky states to blend, and how far toward the second (0..1),
/// from the sun's elevation and whether it is before or after local noon.
/// Overcast and storm are chosen by weather and cross-faded on top of this.
pub fn clear_sky_blend(sun: SkyPos, solar_minute: f32) -> (SkyState, SkyState, f32) {
    let el = sun.elevation.to_degrees();
    let am = solar_minute < 720.0;
    // Key elevations (degrees) at which each baked state is exact.
    const NIGHT: f32 = -12.0;
    const TWILIGHT: f32 = -3.0; // dawn / dusk bake
    const LOW: f32 = 8.0; // golden hour bake (evening); morning side passes through quickly
    const MID: f32 = 32.0; // morning / afternoon bake
    const HIGH: f32 = 62.0; // noon bake
    let seg = |a: f32, b: f32| ((el - a) / (b - a)).clamp(0.0, 1.0);
    if el <= NIGHT {
        (SkyState::Night, SkyState::Night, 0.0)
    } else if el <= TWILIGHT {
        let t = seg(NIGHT, TWILIGHT);
        (SkyState::Night, if am { SkyState::Dawn } else { SkyState::Dusk }, t)
    } else if am {
        if el <= MID {
            (SkyState::Dawn, SkyState::Morning, seg(TWILIGHT, MID))
        } else {
            (SkyState::Morning, SkyState::Noon, seg(MID, HIGH))
        }
    } else if el <= LOW {
        (SkyState::Dusk, SkyState::GoldenHour, seg(TWILIGHT, LOW))
    } else if el <= MID {
        (SkyState::GoldenHour, SkyState::Afternoon, seg(LOW, MID))
    } else {
        (SkyState::Afternoon, SkyState::Noon, seg(MID, HIGH))
    }
}

/// Parse `--clock` text: "YYYY-MM-DD h:mmam", "YYYY-MM-DD ham", or with
/// 24-hour "HH:MM". Returns None if it cannot be read.
pub fn parse_clock(s: &str) -> Option<Moment> {
    let s = s.trim();
    let (date, time) = match s.split_once([' ', 'T']) {
        Some((d, t)) => (d, t.trim()),
        None => (s, "12pm"),
    };
    let mut it = date.split('-');
    let year: i32 = it.next()?.parse().ok()?;
    let month: i32 = it.next()?.parse().ok()?;
    let day: i32 = it.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let t = time.to_ascii_lowercase().replace(' ', "");
    let (body, half) = if let Some(b) = t.strip_suffix("am") {
        (b, Some(false))
    } else if let Some(b) = t.strip_suffix("pm") {
        (b, Some(true))
    } else {
        (t.as_str(), None)
    };
    let (h, m) = match body.split_once(':') {
        Some((h, m)) => (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?),
        None => (body.parse::<i32>().ok()?, 0),
    };
    let h = match half {
        Some(pm) => {
            if !(1..=12).contains(&h) {
                return None;
            }
            (h % 12) + if pm { 12 } else { 0 }
        }
        None => h,
    };
    if !(0..24).contains(&h) || !(0..60).contains(&m) {
        return None;
    }
    Some(Moment::new(year, month, day, (h * 60 + m) as f32))
}

/// Clock text for the HUD and logs: "7am", "4:45pm".
pub fn clock_text(minute_of_day: f32) -> String {
    let total = minute_of_day.rem_euclid(1440.0) as i32;
    let (h, m) = (total / 60, total % 60);
    let half = if h < 12 { "am" } else { "pm" };
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    if m == 0 {
        format!("{}{}", h12, half)
    } else {
        format!("{}:{:02}{}", h12, m, half)
    }
}

/// Sun direction for the globe shader's frame (y = north pole, longitude
/// measured as atan2(x, z)), from the subsolar point.
pub fn globe_sun_dir(m: Moment) -> [f32; 3] {
    let (lat, lon) = m.subsolar();
    let (la, lo) = (lat.to_radians(), lon.to_radians());
    [la.cos() * lo.sin(), la.sin(), la.cos() * lo.cos()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn deg(r: f32) -> f32 {
        r * 180.0 / PI
    }

    #[test]
    fn solstices_and_equinox() {
        let jun = Moment::new(2026, 6, 21, 720.0);
        let dec = Moment::new(2026, 12, 21, 720.0);
        let mar = Moment::new(2026, 3, 20, 720.0);
        assert!((deg(jun.declination()) - 23.44).abs() < 0.3, "{}", deg(jun.declination()));
        assert!((deg(dec.declination()) + 23.44).abs() < 0.3, "{}", deg(dec.declination()));
        assert!(deg(mar.declination()).abs() < 0.6, "{}", deg(mar.declination()));
    }

    #[test]
    fn subsolar_longitude_follows_the_clock() {
        // Noon UTC: sun near Greenwich. 6pm UTC: near 90° W.
        let (_, lon) = Moment::new(2026, 4, 15, 720.0).subsolar();
        assert!(lon.abs() < 1.5, "{lon}");
        let (_, lon) = Moment::new(2026, 4, 15, 1080.0).subsolar();
        assert!((lon + 90.0).abs() < 1.5, "{lon}");
    }

    #[test]
    fn noon_sun_is_south_in_the_north_and_high() {
        // London, late June, about 1pm BST = noon UTC.
        let p = Moment::new(2026, 6, 21, 720.0).sun_at(51.5, 0.0);
        assert!((deg(p.elevation) - 61.9).abs() < 1.0, "{}", deg(p.elevation));
        assert!((deg(p.azimuth) - 180.0).abs() < 3.0, "{}", deg(p.azimuth));
        let d = p.dir();
        assert!(d[2] > 0.0 && d[1] > 0.0); // toward +z (south), above horizon
    }

    #[test]
    fn sun_rises_east_sets_west_and_night_is_night() {
        let site = (40.0, -100.0); // Kansas; local solar noon ≈ 6:40pm UTC
        let rise = Moment::new(2026, 3, 20, 13.0 * 60.0).sun_at(site.0, site.1);
        let set = Moment::new(2026, 3, 21, 0.5 * 60.0).sun_at(site.0, site.1);
        let night = Moment::new(2026, 3, 20, 7.0 * 60.0).sun_at(site.0, site.1);
        assert!(deg(rise.azimuth) > 80.0 && deg(rise.azimuth) < 110.0, "{}", deg(rise.azimuth));
        assert!(deg(set.azimuth) > 250.0 && deg(set.azimuth) < 280.0, "{}", deg(set.azimuth));
        assert!(deg(night.elevation) < -30.0, "{}", deg(night.elevation));
    }

    #[test]
    fn moon_phase_known_dates() {
        // Full moon 2026-03-03, new moon 2026-03-19 (UTC).
        let full = Moment::new(2026, 3, 3, 700.0).moon_phase();
        let new = Moment::new(2026, 3, 19, 100.0).moon_phase();
        assert!((full - 0.5).abs() < 0.04, "{full}");
        assert!(new < 0.04 || new > 0.96, "{new}");
    }

    #[test]
    fn clock_parsing_and_text() {
        let m = parse_clock("2026-07-04 4:45pm").unwrap();
        assert_eq!((m.year, m.month, m.day, m.minute), (2026, 7, 4, 1005.0));
        assert_eq!(parse_clock("2026-07-04 7am").unwrap().minute, 420.0);
        assert_eq!(parse_clock("2026-07-04 12am").unwrap().minute, 0.0);
        assert_eq!(parse_clock("2026-07-04 16:45").unwrap().minute, 1005.0);
        assert!(parse_clock("yesterday").is_none());
        assert_eq!(clock_text(1005.0), "4:45pm");
        assert_eq!(clock_text(420.0), "7am");
        assert_eq!(clock_text(0.0), "12am");
    }

    #[test]
    fn blend_walks_the_day_in_order() {
        let site = (45.0, 0.0);
        let mut seen = Vec::new();
        for i in 0..96 {
            let m = Moment::new(2026, 6, 21, i as f32 * 15.0);
            let (a, b, t) = clear_sky_blend(m.sun_at(site.0, site.1), m.solar_minute(site.1));
            assert!((0.0..=1.0).contains(&t));
            let s = if t < 0.5 { a } else { b };
            if seen.last() != Some(&s) {
                seen.push(s);
            }
        }
        use SkyState::*;
        assert_eq!(seen, vec![Night, Dawn, Morning, Noon, Afternoon, GoldenHour, Dusk, Night]);
    }
}
