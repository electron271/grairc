/*
https://git.sr.ht/~sheepy/cabbage/tree/main/item/src/gfx/mod.rs

Copyright © 2025 Ryan Sylvia <sheepy@sheepy.moe>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use ctru::{
    console::ConsoleScreen,
    prelude::Gfx,
    services::{
        gfx::{BottomScreen, Flush, Screen, Swap, TopScreen},
        gspgpu::FramebufferFormat,
    },
};
use std::cell::RefCell;

pub mod eg;

pub struct DisplayTargets<'a> {
    pub top: Display<'a, TopScreen, 400, 240>,
    pub bottom: Display<'a, BottomScreen, 320, 240>,
}
impl<'a> DisplayTargets<'a> {
    pub fn new(gfx: &'a Gfx) -> anyhow::Result<Self> {
        let top = Display::new(&gfx.top_screen)?;
        let bottom = Display::new(&gfx.bottom_screen)?;
        Ok(Self { top, bottom })
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        let top = self.top.flush();
        let bottom = self.bottom.flush();
        top?;
        bottom?;
        Ok(())
    }
}

/// N.B. `W` and `H` are going by a horizontal measurement such that W > H (i.e. not rotated 90deg)
pub struct Display<'a, C: ConsoleScreen, const W: u32, const H: u32> {
    pub framebuffer: &'a mut [u8],
    display: &'a RefCell<C>,
    /// If a display is "disabled," all methods will successfully exit early.
    /// This is useful to use a display as a console for debugging.
    pub disabled: bool,
}

pub trait FbDisplay {
    fn is_disabled(&self) -> bool;
    fn flush(&mut self) -> anyhow::Result<()>;
}
impl<'a, C: Swap + Screen + Flush, const W: u32, const H: u32> Display<'a, C, W, H> {
    pub fn new(display: &'a RefCell<C>) -> anyhow::Result<Self> {
        let mut display_mut = display.try_borrow_mut()?;
        display_mut.set_double_buffering(false);
        display_mut.set_framebuffer_format(FramebufferFormat::Bgr8);

        let (framebuffer, width, height) = unsafe {
            let fb = display_mut.raw_framebuffer();
            (
                std::slice::from_raw_parts_mut(fb.ptr, fb.width * fb.height * 3),
                // n.b. W and H are swapped because the fb is rotated 90deg
                fb.height as u32,
                fb.width as u32,
            )
        };
        assert!(width == W);
        assert!(height == H);

        Ok(Self {
            display,
            framebuffer,
            disabled: false,
        })
    }

    /// Swap and flush the buffers of the borrowed display reference
    fn _flush(&mut self) -> anyhow::Result<()> {
        if self.disabled {
            return Ok(());
        }

        let mut display = self.display.try_borrow_mut()?;

        display.flush_buffers();
        display.swap_buffers();

        let fb = display.raw_framebuffer();
        self.framebuffer = unsafe { std::slice::from_raw_parts_mut(fb.ptr, (W * H * 3) as _) };

        Ok(())
    }
}

/// Bottom screen
impl FbDisplay for Display<'_, BottomScreen, 320, 240> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        if self.is_disabled() {
            return Ok(());
        }

        {
            let mut display = self.display.try_borrow_mut()?;
            display.set_framebuffer_format(FramebufferFormat::Bgr8);
            display.set_double_buffering(true);
        }

        self._flush()?;
        Ok(())
    }
}

/// Top screen
impl FbDisplay for Display<'_, TopScreen, 400, 240> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        if self.is_disabled() {
            return Ok(());
        }

        {
            let mut display = self.display.try_borrow_mut()?;
            display.set_framebuffer_format(FramebufferFormat::Bgr8);
            display.set_double_buffering(true);
            // Entering console mode can set this to true
            display.set_wide_mode(false);
        }
        self._flush()?;
        Ok(())
    }
}
