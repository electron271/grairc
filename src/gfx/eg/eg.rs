/*
https://git.sr.ht/~sheepy/cabbage/tree/main/item/src/gfx/eg.rs

Copyright © 2025 Ryan Sylvia <sheepy@sheepy.moe>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use super::Display;

use ctru::console::ConsoleScreen;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

impl<C: ConsoleScreen, const W: u32, const H: u32> DrawTarget for Display<'_, C, W, H> {
    /// The framebuffer is *actually* Brg888!!
    type Color = Rgb888;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        if self.disabled {
            return Ok(());
        }

        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0_u32.., y @ 0_u32..)) = coord.try_into() {
                if x >= W || y >= H {
                    continue;
                }

                // Bottom to top, left to right
                let index = 3 * (x * H + (H - 1 - y));
                let index = index as usize;
                // Bgr888 color space!
                self.framebuffer[index] = color.b();
                self.framebuffer[index + 1] = color.g();
                self.framebuffer[index + 2] = color.r();
            }
        }

        Ok(())
    }
}

impl<C: ConsoleScreen, const W: u32, const H: u32> OriginDimensions for Display<'_, C, W, H> {
    fn size(&self) -> Size {
        Size::new(W, H)
    }
}
