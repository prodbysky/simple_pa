# pulseaudio-simple-rs
- Idiomatic Rust wrapper around `pa_simple_*`  

A small, minimal, and type-safe Rust wrapper around the PulseAudio Simple API (`libpulse-simple`).  
This crate provides an ergonomic interface for basic audio playback without requiring the full asynchronous PulseAudio API.

Status: Experimental - covers only the Simple API.  
Use case: Quick audio output/input where low latency or advanced routing is not required.

## Requirements

PulseAudio development headers must be installed.

Debian/Ubuntu:
```
sudo apt install libpulse-dev
```

Fedora:
```
sudo dnf install pulseaudio-libs-devel
```

Arch Linux:
```
sudo pacman -S pulseaudio
```

## Add to Cargo.toml

```
[dependencies]
pulseaudio-simple = "0.1"
```

## Example: Playback (i16 stereo, 44.1 kHz)

```rust
use pulseaudio_simple::{Simple, SampleSpec, StreamDirection};

fn main() -> Result<(), String> {
    let spec = SampleSpec::new(44_100, 2);

    let mut pa = Simple::<i16>::new(
        "example-playback",
        StreamDirection::Playback,
        spec,
    )?;

    let mut buffer = [0i16; 44100];
    for (i, sample) in buffer.iter_mut().enumerate() {
        let t = i as f32 / 44100.0;
        *sample = (t * 440.0 * std::f32::consts::TAU).sin() as f32 as i16;
    }

    pa.write(&mut buffer)?;
    pa.drain()?;

    Ok(())
}
```

## Safety Notes

- Uses `unsafe` FFI calls internally.  
- Safe API ensures:
  - RAII cleanup via `Drop`
  - Correct lifetimes for C strings and structures  

## TODO
 - [ ] Capture API

## License
MIT
