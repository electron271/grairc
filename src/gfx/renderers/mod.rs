use anyhow::Error;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Text};
use tinytga::Tga;

use crate::gfx::eg::DisplayTargets;
use crate::state::State;

pub fn wrap(text: &str, width: usize) -> String {
    // TODO: better word wrapping
    let mut wrapped = String::new();
    let mut line_length = 0;
    for c in text.chars() {
        if line_length >= width {
            wrapped.push('\n');
            line_length = 0;
        } else {
            wrapped.push(c);
            line_length += 1;
        }
    }
    wrapped
}

pub fn render_info(targets: &mut DisplayTargets) -> Result<(), Error> {
    let text = format!(
        "grairc v{}
START  > exit
DPAD   > switch channels
A      > send message
X      > send raw irc command
",
        env!("CARGO_PKG_VERSION")
    );

    Text::with_alignment(
        &text,
        targets.bottom.bounding_box().top_left + Point::new(5, 10),
        MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE),
        Alignment::Left,
    )
    .draw(&mut targets.bottom)?;

    Ok(())
}

pub fn render_channels(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    let mut next = Point::new(
        (targets.bottom.bounding_box().size.width - 5)
            .try_into()
            .unwrap(),
        10,
    );
    for channel in &state.channels {
        let text = format!("{}\n", channel.name);

        let mut style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        if channel.selected {
            style.background_color = Some(Rgb888::WHITE);
            style.text_color = Some(Rgb888::BLACK);
        } else {
            style.background_color = Some(Rgb888::BLACK);
            style.text_color = Some(Rgb888::WHITE);
        }

        next =
            Text::with_alignment(&text, next, style, Alignment::Right).draw(&mut targets.bottom)?;
    }

    Ok(())
}

pub fn render_bar(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    let bar_height = 16;
    let bar_area = Rectangle::new(
        Point::new(
            0,
            (targets.top.bounding_box().size.height - bar_height)
                .try_into()
                .unwrap(),
        ),
        Size::new(targets.top.bounding_box().size.width, bar_height),
    );
    bar_area
        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
        .draw(&mut targets.top)?;

    let battery_charging = include_bytes!("../../../assets/battery/charging.tga");
    let battery_critical = include_bytes!("../../../assets/battery/critical.tga");
    let battery_drained = include_bytes!("../../../assets/battery/drained.tga");
    let battery_high = include_bytes!("../../../assets/battery/high.tga");
    let battery_low = include_bytes!("../../../assets/battery/low.tga");
    let battery_medium = include_bytes!("../../../assets/battery/medium.tga");
    let battery_verylow = include_bytes!("../../../assets/battery/verylow.tga");
    let battery_image: Tga<Rgb888> = Tga::from_slice(match state.battery_state {
        crate::state::BatteryState::Charging => battery_charging,
        crate::state::BatteryState::Critical => battery_critical,
        crate::state::BatteryState::Drained => battery_drained,
        crate::state::BatteryState::VeryLow => battery_verylow,
        crate::state::BatteryState::Low => battery_low,
        crate::state::BatteryState::Medium => battery_medium,
        crate::state::BatteryState::High => battery_high,
    })
    .unwrap();
    Image::new(
        &battery_image,
        targets.top.bounding_box().bottom_right().unwrap()
            - Point::new(16, (bar_height - 1).try_into().unwrap()),
    )
    .draw(&mut targets.top)?;

    // this is actually local time because of some 3ds shit
    let cur_time = time::OffsetDateTime::now_utc();
    let time_text = format!(
        "{:02}:{:02}:{:02}",
        cur_time.hour(),
        cur_time.minute(),
        cur_time.second()
    );
    Text::with_alignment(
        &time_text,
        targets.top.bounding_box().bottom_right().unwrap() - Point::new(28, 5),
        MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK),
        Alignment::Right,
    )
    .draw(&mut targets.top)?;

    let binding = crate::config::Config::default();
    let config = state.config.as_ref().unwrap_or(&binding);
    Text::with_alignment(
        &config.nickname,
        Point::new(5, targets.top.bounding_box().size.height as i32 - 5),
        MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK),
        Alignment::Left,
    )
    .draw(&mut targets.top)?;

    Ok(())
}

pub fn render_user_list(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    let user_list_area = Rectangle::new(
        Point::new(
            (targets.top.bounding_box().size.width - 80)
                .try_into()
                .unwrap(),
            0,
        ),
        Size::new(80, targets.top.bounding_box().size.height - 16),
    );
    user_list_area
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(24, 24, 24)))
        .draw(&mut targets.top)?;

    let current_channel = state.current_channel_static();
    let mut next = Point::new(
        (targets.top.bounding_box().size.width - 75)
            .try_into()
            .unwrap(),
        10,
    );
    for user in &current_channel.users {
        let text = format!("{}\n", user);
        next = Text::with_alignment(
            &text,
            next,
            MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE),
            Alignment::Left,
        )
        .draw(&mut targets.top)?;
    }
    Ok(())
}

pub fn render_messages(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    let message_area = Rectangle::new(
        Point::new(0, 0),
        Size::new(
            (targets.top.bounding_box().size.width - 80)
                .try_into()
                .unwrap(),
            (targets.top.bounding_box().size.height - 16)
                .try_into()
                .unwrap(),
        ),
    );
    message_area
        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLACK))
        .draw(&mut targets.top)?;

    let current_channel = state.current_channel_static();
    let mut next = Point::new(5, 10);
    for message in &current_channel.messages {
        let mut text = if let Some(nick) = &message.nick {
            let nick_text = format!("{:12}:", nick);
            let mut nick_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
            nick_style.background_color = Some(Rgb888::WHITE);
            next = Text::with_alignment(&nick_text, next, nick_style, Alignment::Left)
                .draw(&mut targets.top)?;
            format!(" {}\n", message.content)
        } else {
            format!("{}\n", message.content)
        };

        text = wrap(&text, 50);

        next = Text::with_alignment(
            &text,
            next,
            MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE),
            Alignment::Left,
        )
        .draw(&mut targets.top)?;

        next.x = 5;
    }
    Ok(())
}

pub fn render_main_screen(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    render_messages(targets, state)?;
    render_bar(targets, state)?;
    render_user_list(targets, state)?;
    Ok(())
}

pub fn render(targets: &mut DisplayTargets, state: &State) -> Result<(), Error> {
    render_info(targets)?;
    render_channels(targets, state)?;
    render_main_screen(targets, state)?;
    Ok(())
}
