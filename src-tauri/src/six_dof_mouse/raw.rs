//! Existing HID decoding/coalescing, independent of a window or event emitter.
use super::*;

pub(super) fn spawn(
    mut read: impl FnMut(&mut [u8], i32) -> Result<usize, String> + Send + 'static,
    sink: SixDofEventSink,
) -> Result<RawHidWorker, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    let thread = std::thread::Builder::new()
        .name("nbcad-six-dof-mouse".to_string())
        .spawn(move || {
            let mut buffer = [0_u8; 64];
            let mut reports = Reports::new(Instant::now());
            while !worker_stop.load(Ordering::Relaxed) {
                let length = match read(&mut buffer, RAW_HID_READ_TIMEOUT_MS) {
                    Ok(length) => length,
                    Err(error) => {
                        sink(SixDofEvent::Error(error));
                        break;
                    }
                };
                reports.receive(&buffer[..length], &sink);
                reports.flush(Instant::now, &sink);
            }
        })
        .map_err(|error| error.to_string())?;
    Ok(RawHidWorker {
        stop,
        thread: Some(thread),
    })
}

struct Reports {
    previous_buttons: u32,
    translation: Option<[i16; 3]>,
    rotation: Option<[i16; 3]>,
    last_emit: Instant,
}

impl Reports {
    fn new(now: Instant) -> Self {
        Self {
            previous_buttons: 0,
            translation: None,
            rotation: None,
            last_emit: now.checked_sub(RAW_HID_EMIT_INTERVAL).unwrap_or(now),
        }
    }

    fn receive(&mut self, report: &[u8], sink: &SixDofEventSink) {
        if report.len() < 2 {
            return;
        }
        let data = &report[1..];
        match report[0] {
            1 => {
                if let Some(translation) = vector(data, 0) {
                    self.translation = Some(translation);
                }
                if let Some(rotation) = vector(data, 6) {
                    self.rotation = Some(rotation);
                }
            }
            2 => {
                if let Some(rotation) = vector(data, 0) {
                    self.rotation = Some(rotation);
                }
            }
            3 => {
                let mut bytes = [0_u8; 4];
                let count = data.len().min(bytes.len());
                bytes[..count].copy_from_slice(&data[..count]);
                let buttons = u32::from_le_bytes(bytes);
                let newly_pressed = buttons & !self.previous_buttons;
                self.previous_buttons = buttons;
                for index in 0..32 {
                    if newly_pressed & (1 << index) != 0 {
                        sink(SixDofEvent::Button(ButtonPacket { button: index + 1 }));
                    }
                }
            }
            _ => {}
        }
    }

    fn flush(&mut self, mut now: impl FnMut() -> Instant, sink: &SixDofEventSink) {
        if (self.translation.is_some() || self.rotation.is_some())
            && now().duration_since(self.last_emit) >= RAW_HID_EMIT_INTERVAL
        {
            sink(SixDofEvent::Motion(MotionPacket {
                translation: self.translation.take(),
                rotation: self.rotation.take(),
            }));
            self.last_emit = now();
        }
    }
}

#[cfg(test)]
mod tests;
