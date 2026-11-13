use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

#[derive(Clone)]
pub struct SoundEffects {
    pub rifle: Option<Sound>,
    pub cored: Option<Sound>,
    pub arc: Option<Sound>,
    pub explosion: Option<Sound>,
    pub demolish: Option<Sound>,
    pub screech: Option<Sound>,
    pub click: Option<Sound>,
    pub reload: Option<Sound>,
    pub scream: Option<Sound>,
    pub equip: Option<Sound>,
    pub psi: Option<Sound>,
    // Underwater sound set
    pub uw_rifle: Option<Sound>,
    pub uw_cav: Option<Sound>,
    pub uw_pressure: Option<Sound>,
    pub uw_explosion: Option<Sound>,
    pub uw_screech: Option<Sound>,
    pub sonar: Option<Sound>,
    pub bubbles: Option<Sound>,
}

impl SoundEffects {
    pub async fn load() -> Self {
        let rifle = load_wav(&gen_rifle()).await;
        let cored = load_wav(&gen_cored()).await;
        let arc = load_wav(&gen_arc()).await;
        let explosion = load_wav(&gen_explosion()).await;
        let demolish = load_wav(&gen_demolish()).await;
        let screech = load_wav(&gen_screech()).await;
        let click = load_wav(&gen_click()).await;
        let reload = load_wav(&gen_reload()).await;
        let scream = load_wav(&gen_scream()).await;
        let equip = load_wav(&gen_equip()).await;
        let psi = load_wav(&gen_psi()).await;

        let uw_rifle = load_wav(&gen_uw_rifle()).await;
        let uw_cav = load_wav(&gen_uw_cav()).await;
        let uw_pressure = load_wav(&gen_uw_pressure()).await;
        let uw_explosion = load_wav(&gen_uw_explosion()).await;
        let uw_screech = load_wav(&gen_uw_screech()).await;
        let sonar = load_wav(&gen_sonar()).await;
        let bubbles = load_wav(&gen_bubbles()).await;

        Self {
            rifle, cored, arc, explosion, demolish, screech, click, reload, scream, equip, psi,
            uw_rifle, uw_cav, uw_pressure, uw_explosion, uw_screech, sonar, bubbles,
        }
    }

    pub fn play_rifle(&self) { if let Some(s) = &self.rifle { play_sound_once(s); } }
    pub fn play_cored(&self) { if let Some(s) = &self.cored { play_sound_once(s); } }
    pub fn play_arc(&self) { if let Some(s) = &self.arc { play_sound_once(s); } }
    pub fn play_explosion(&self) { if let Some(s) = &self.explosion { play_sound_once(s); } }
    pub fn play_demolish(&self) { if let Some(s) = &self.demolish { play_sound_once(s); } }
    pub fn play_screech(&self) { if let Some(s) = &self.screech { play_sound_once(s); } }
    pub fn play_click(&self) { if let Some(s) = &self.click { play_sound_once(s); } }
    pub fn play_reload(&self) { if let Some(s) = &self.reload { play_sound_once(s); } }
    pub fn play_scream(&self) { if let Some(s) = &self.scream { play_sound_once(s); } }
    pub fn play_equip(&self) { if let Some(s) = &self.equip { play_sound_once(s); } }
    pub fn play_psi(&self) { if let Some(s) = &self.psi { play_sound_once(s); } }

    pub fn play_uw_rifle(&self) { if let Some(s) = &self.uw_rifle { play_sound_once(s); } else { self.play_rifle(); } }
    pub fn play_uw_cav(&self) { if let Some(s) = &self.uw_cav { play_sound_once(s); } else { self.play_cored(); } }
    pub fn play_uw_pressure(&self) { if let Some(s) = &self.uw_pressure { play_sound_once(s); } else { self.play_arc(); } }
    pub fn play_uw_explosion(&self) { if let Some(s) = &self.uw_explosion { play_sound_once(s); } else { self.play_explosion(); } }
    pub fn play_uw_screech(&self) { if let Some(s) = &self.uw_screech { play_sound_once(s); } else { self.play_screech(); } }
    pub fn play_sonar(&self) { if let Some(s) = &self.sonar { play_sound_once(s); } }
    pub fn play_bubbles(&self) { if let Some(s) = &self.bubbles { play_sound_once(s); } }
}

async fn load_wav(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}

/// Encode raw 16-bit signed PCM mono samples into standard WAV format.
fn make_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    let file_len = data_len + 36;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&file_len.to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
    out.extend_from_slice(&1u16.to_le_bytes());  // PCM
    out.extend_from_slice(&1u16.to_le_bytes());  // Mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes());  // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

fn gen_rifle() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.14) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0x12345678u32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 38.0).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        let thump = (t * 140.0 * std::f32::consts::TAU).sin() * (-t * 22.0).exp();
        let val = ((noise * 0.75 + thump * 0.65) * env * 24000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_cored() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.16) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 18.0).exp();
        let freq = 1800.0 * (1.0 - t / 0.16) + 320.0;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let sine = phase.sin();
        let square = if sine > 0.0 { 0.7 } else { -0.7 };
        let val = ((sine * 0.6 + square * 0.4) * env * 22000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_arc() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.22) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 14.0).exp();
        let freq = 820.0 * (1.0 - (t / 0.22).powi(2)) + 140.0;
        let mod_phase = (t * 45.0 * std::f32::consts::TAU).sin() * 2.5;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let val = ((phase + mod_phase).sin() * env * 26000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_explosion() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.45) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0xdeadbeefu32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 9.5).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        let rumble = (t * 65.0 * std::f32::consts::TAU).sin() * (-t * 7.0).exp();
        let val = ((noise * 0.85 + rumble * 0.5) * env * 28000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_demolish() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.38) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0x55aa1234u32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 11.0).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        let crunch = if (i % 120) < 60 { noise * 1.2 } else { -noise * 0.8 };
        let val = (crunch * env * 26000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_screech() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.32) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 10.0).exp() * (1.0 - (-t * 60.0).exp());
        let freq = 1100.0 + (t * 30.0 * std::f32::consts::TAU).sin() * 350.0 - t * 400.0;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let val = (phase.sin() * env * 22000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_click() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.04) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 120.0).exp();
        let sine = (t * 950.0 * std::f32::consts::TAU).sin();
        let val = (sine * env * 18000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_reload() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.18) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let click1 = if t < 0.05 { (-(t) * 80.0).exp() * (t * 800.0 * std::f32::consts::TAU).sin() } else { 0.0 };
        let click2 = if t > 0.09 && t < 0.16 { (-(t - 0.09) * 70.0).exp() * ((t - 0.09) * 1100.0 * std::f32::consts::TAU).sin() } else { 0.0 };
        let val = ((click1 + click2) * 20000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_scream() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.35) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 6.0).exp() * (1.0 - (-t * 40.0).exp());
        let freq = 650.0 - t * 350.0 + (t * 18.0 * std::f32::consts::TAU).sin() * 80.0;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let val = (phase.sin() * env * 23000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_equip() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.09) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0x87654321u32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 55.0).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        let click = (t * 450.0 * std::f32::consts::TAU).sin() * (-t * 70.0).exp();
        let val = ((noise * 0.35 + click * 0.75) * env * 22000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

fn gen_psi() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.45) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-(t - 0.22).powi(2) * 22.0).exp();
        let lfo = (t * 26.0 * std::f32::consts::TAU).sin() * 75.0;
        let freq = 520.0 + (t * 4.0 * std::f32::consts::TAU).sin() * 110.0 + lfo;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let harmonic = (phase * 1.5).sin() * 0.35;
        let val = ((phase.sin() * 0.75 + harmonic) * env * 24000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

// =========================================================================
// UNDERWATER ACOUSTIC FILTERED SOUNDS (underwater campaign)
// =========================================================================

/// Underwater dive pistol / dive rifle: compressed gas ejection with hydrodynamic water rush.
fn gen_uw_rifle() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.22) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0x33445566u32;
    let mut lpf = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 24.0).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        // Low-pass filter noise for submerged hydrodynamic whoosh
        lpf += 0.12 * (noise - lpf);
        let sub_thump = (t * 75.0 * std::f32::consts::TAU).sin() * (-t * 16.0).exp();
        let val = ((lpf * 1.1 + sub_thump * 0.9) * env * 25000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Underwater cav weapons: steam boiling pop and cavitation collapse.
fn gen_uw_cav() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.20) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 16.0).exp();
        let freq = 680.0 * (1.0 - t / 0.20) + 160.0;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let bubble_mod = (t * 85.0 * std::f32::consts::TAU).sin() * 0.4;
        let val = ((phase.sin() + bubble_mod) * env * 24000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Underwater pressure tools: cavitation weapon.
fn gen_uw_pressure() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.28) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 11.0).exp();
        let freq = 340.0 * (1.0 - (t / 0.28).powi(2)) + 95.0;
        let mod_phase = (t * 32.0 * std::f32::consts::TAU).sin() * 3.2;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let val = ((phase + mod_phase).sin() * env * 27000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Submerged depth charge / heavy hydrostatic torpedo explosion.
fn gen_uw_explosion() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.58) as usize;
    let mut s = Vec::with_capacity(n);
    let mut seed = 0x99887766u32;
    let mut lpf = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 6.5).exp();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as i16) as f32 / 32768.0;
        // Severe acoustic dampening for deep water mass displacement
        lpf += 0.08 * (noise - lpf);
        let shockwave = (t * 44.0 * std::f32::consts::TAU).sin() * (-t * 4.2).exp();
        let resonance = (t * 88.0 * std::f32::consts::TAU).sin() * (-t * 7.0).exp();
        let val = ((lpf * 0.8 + shockwave * 1.2 + resonance * 0.4) * env * 29000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Underwater creature call (Fathom / Mantle).
fn gen_uw_screech() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.38) as usize;
    let mut s = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = (-t * 7.5).exp() * (1.0 - (-t * 35.0).exp());
        let freq = 540.0 + (t * 18.0 * std::f32::consts::TAU).sin() * 160.0 - t * 200.0;
        phase += freq / rate as f32 * std::f32::consts::TAU;
        let val = (phase.sin() * env * 23000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Active sonar ping.
fn gen_sonar() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.65) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let ping = (t * 1180.0 * std::f32::consts::TAU).sin() * (-t * 5.2).exp();
        // Oceanic delayed echo reflection
        let echo = if t > 0.18 {
            let te = t - 0.18;
            (te * 1180.0 * std::f32::consts::TAU).sin() * (-te * 6.5).exp() * 0.28
        } else {
            0.0
        };
        let val = ((ping + echo) * 23000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}

/// Diver exhalation respirator bubbles and gurgle.
fn gen_bubbles() -> Vec<u8> {
    let rate = 22050;
    let n = (rate as f32 * 0.38) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let mut total = 0.0f32;
        // Staggered ascending bubble pops
        for (delay, freq) in [(0.02, 360.0), (0.08, 520.0), (0.15, 680.0), (0.21, 440.0), (0.28, 760.0)] {
            if t > delay && t < delay + 0.08 {
                let dt = t - delay;
                let env = (-(dt - 0.025).powi(2) * 800.0).exp();
                total += (dt * freq * std::f32::consts::TAU).sin() * env * 0.35;
            }
        }
        let val = (total * 22000.0).clamp(-32000.0, 32000.0) as i16;
        s.push(val);
    }
    make_wav(&s, rate)
}
