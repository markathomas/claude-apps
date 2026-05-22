use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    pub out_time_ms: u64,
    pub fps: Option<f32>,
}

pub struct ProgressParser {
    out_time_re: Regex,
    fps_re: Regex,
    last_out_time_ms: u64,
    last_fps: Option<f32>,
}

impl Default for ProgressParser {
    fn default() -> Self {
        Self {
            out_time_re: Regex::new(r"out_time_ms=([0-9]+)").unwrap(),
            fps_re: Regex::new(r"\bfps=\s*([0-9.]+)").unwrap(),
            last_out_time_ms: 0,
            last_fps: None,
        }
    }
}

impl ProgressParser {
    pub fn feed(&mut self, line: &str) -> Option<Progress> {
        let mut updated = false;
        if let Some(caps) = self.out_time_re.captures(line) {
            if let Ok(us) = caps[1].parse::<u64>() {
                self.last_out_time_ms = us / 1000;
                updated = true;
            }
        }
        if let Some(caps) = self.fps_re.captures(line) {
            if let Ok(f) = caps[1].parse::<f32>() {
                self.last_fps = Some(f);
            }
        }
        if updated {
            Some(Progress {
                out_time_ms: self.last_out_time_ms,
                fps: self.last_fps,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_out_time_ms_microseconds_to_ms() {
        let mut p = ProgressParser::default();
        let progress = p.feed("out_time_ms=1500000").unwrap();
        assert_eq!(progress.out_time_ms, 1500);
    }

    #[test]
    fn parses_fps_from_combined_line() {
        let mut p = ProgressParser::default();
        let line = "frame=  120 fps= 30 q=28.0 size=    256kB time=00:00:04.00 bitrate=...";
        // No out_time_ms in this line — no Progress emitted
        assert!(p.feed(line).is_none());
        // But fps was captured for next time
        assert_eq!(p.last_fps, Some(30.0));
    }

    #[test]
    fn returns_none_for_irrelevant_lines() {
        let mut p = ProgressParser::default();
        assert!(p.feed("Stream #0:0 -> #0:0").is_none());
        assert!(p.feed("[libx264] frame I").is_none());
    }

    #[test]
    fn carries_fps_into_subsequent_progress() {
        let mut p = ProgressParser::default();
        p.feed("frame=10 fps=24 time=00:00:00.40");
        let progress = p.feed("out_time_ms=400000").unwrap();
        assert_eq!(progress.out_time_ms, 400);
        assert_eq!(progress.fps, Some(24.0));
    }
}
