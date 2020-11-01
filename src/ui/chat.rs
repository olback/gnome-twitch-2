use {
    crate::{
        ASSETS, USER, p, get_obj, debug, warning, rt, error::GtResult,
        ui::show_info_bar, resources::bytes_to_pixbuf
    },
    std::{rc::Rc, cell::RefCell},
    twitchchat::{
        commands, UserConfig,
        connector::tokio::ConnectorNativeTls,
        messages::Commands,
        runner::{AsyncRunner, Status, NotifyHandle},
        writer::{AsyncWriter, MpscWriter}
    },
    gtk::{
        Builder, Button, Entry, TextView, TextBuffer, TextTag, TextIter,
        TextTagTable, ScrolledWindow, prelude::*
    },
    gdk_pixbuf::Pixbuf,
    glib::{clone, Sender}
};

#[derive(Debug)]
pub enum MessagePart {
    Text(String),
    Pixbuf(Pixbuf)
}

type RGB = (u8, u8, u8);

// Stolen from https://github.com/vinszent/gnome-twitch/blob/master/src/gt-chat.c#L43
const DEFAULT_CHAT_COLORS: &'static [RGB] = &[
    (0xff, 0x00, 0x00), (0x00, 0x00, 0xff), (0x00, 0xff, 0x00), (0xb2, 022, 0x22),
    (0xff, 0x7f, 0x50), (0x9a, 0xcd, 0x32), (0xff, 0x45, 0x00), (0x2e, 0x8b, 0x57),
    (0xda, 0xa5, 0x20), (0xd2, 0x69, 0x1e), (0x5f, 0x9e, 0xa0), (0x1e, 0x90, 0xff),
    (0xff, 0x69, 0xb4), (0x8a, 0x2b, 0xe2), (0x00, 0xff, 0x7f)
];

pub struct ChatSection {
    input: Entry,
    view: TextView,
    scroll: ScrolledWindow,
    buffer: TextBuffer,
    tag_table: TextTagTable,
    buffer_tx: Sender<Commands<'static>>,
    writer: Rc<RefCell<Option<AsyncWriter<MpscWriter>>>>,
    quit_handle: Rc<RefCell<Option<(NotifyHandle, NotifyHandle)>>>,
    channel: RefCell<String>
}

impl ChatSection {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        let (buffer_tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let inner = Rc::new(Self {
            input: get_obj!(builder, "chat-input"),
            view: get_obj!(builder, "chat-view"),
            scroll: get_obj!(builder, "chat-scroll"),
            buffer: get_obj!(builder, "chat-buffer"),
            tag_table: get_obj!(builder, "chat-tag-table"),
            buffer_tx,
            writer: Rc::new(RefCell::new(None)),
            quit_handle: Rc::new(RefCell::new(None)),
            channel: RefCell::new(String::new())
        });

        inner.view.connect_size_allocate(clone!(@strong inner => move |_, _| {
            inner.scroll_bottom()
        }));

        rx.attach(None, clone!(@strong inner => move |command| {

            // debug!("{:#?}", command);

            match command {
                Commands::RoomState(room_state) => {
                    warning!("TODO {:#?}", room_state)
                },
                // Commands::ClearChat(_) => {
                //     inner.clear_chat()
                // },
                Commands::Notice(notice) => {
                    inner.append_notice(notice.message())
                },
                Commands::UserNotice(user_notice) => if let Some(message) = user_notice.message() {
                    inner.append_notice(message)
                },
                Commands::Privmsg(privmsg) => {
                    // TODO: Handle Badges and cheers.
                    let from = privmsg.display_name().unwrap_or(privmsg.name()).to_string();
                    let color = privmsg.color()
                        .map(|c| (c.rgb.red(), c.rgb.green(), c.rgb.blue()))
                        .unwrap_or(get_color(&from));
                    let body = String::from(privmsg.data());
                    let badges = privmsg.badges();
                    let emotes = privmsg.emotes();

                    rt::run_cb_local(async move {
                        let mut emotes_data = Vec::<(usize, Vec<u8>)>::with_capacity(emotes.len());
                        for emote in emotes {
                            match ASSETS.load(&format!("https://static-cdn.jtvnw.net/emoticons/v1/{}/1.0", emote.id)).await {
                                Ok(data) => emotes_data.push((emote.id, data)),
                                Err(e) => warning!("{:#?}", e)
                            }
                        }
                        emotes_data
                    }, clone!(@strong inner, @strong privmsg => move |emotes_data| {

                        // FIXME:TODO: This almost always fails, why?
                        let emotes_id_pixbuf = emotes_data
                            .into_iter()
                            .filter_map(|(id, data)| match bytes_to_pixbuf(&data, Some((18, 18))) {
                                Ok(p) => Some((id, p)),
                                Err(e) => {
                                    warning!("{:#?}", e);
                                    None
                                }
                            })
                            .collect::<Vec::<(usize, Pixbuf)>>();

                        debug!("{:#?}", emotes_id_pixbuf);

                        let mut emotes = Vec::<((usize, usize), Pixbuf)>::new();

                        for emote in privmsg.emotes() {
                            for (loaded_emote_id, loaded_emote_pixbuf) in &emotes_id_pixbuf {
                                if &emote.id == loaded_emote_id {
                                    for emote_range in emote.ranges {
                                        emotes.push((
                                            (emote_range.start as usize, emote_range.end as usize),
                                            loaded_emote_pixbuf.clone()
                                        ))
                                    }
                                    break
                                }
                            }
                        }

                        emotes.sort_by(|a, b| a.0.0.cmp(&b.0.0));
                        let msg_parts = partify(body.clone(), emotes);

                        inner.append_message(
                            &[] as &[&Pixbuf],
                            color,
                            &from,
                            msg_parts
                        )

                    }));

                }
                _ => { /* Unhandled message, do nothing */ }
            }

            glib::Continue(true)

        }));

        inner.input.connect_activate(clone!(@strong inner => move |entry| {

            let channel = (&*inner.channel.borrow()).clone();
            let text = entry.get_text().to_string().trim().to_string();

            if text.is_empty() {
                return
            }

            if let Some(mut writer) = inner.writer.borrow().clone() {
                rt::run_cb_local(async move {
                    writer.encode(commands::privmsg(&channel, &text)).await
                }, clone!(@strong inner => move |res| {
                    if let Err(e) = res {
                        show_info_bar(
                            "Chat error",
                            &e.to_string(),
                            None::<&gtk::Widget>,
                            gtk::MessageType::Error
                        );
                    } else {
                        inner.input.set_text("");
                    }
                }));
            }

        }));

        get_obj!(Button, builder, "chat-clear").connect_clicked(clone!(@strong inner => move |_| {
            inner.clear_chat()
        }));

        inner

    }

    pub fn append_message<B: AsRef<Pixbuf>>(
        &self,
        badges: &[B],
        color: RGB,
        from: &str,
        msg_parts: Vec<MessagePart>
    ) {

        // TODO: Links
        // MessagePart
        // let mut parts = Vec::<MessagePart>::new();

        // Badges
        for badge in badges {
            self.buffer.insert_pixbuf(
                &mut self.buffer.get_end_iter(),
                badge.as_ref()
            );
            self.buffer.insert(
                &mut self.buffer.get_end_iter(),
                " "
            );
        }

        let rgb_str = format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);
        let color_tag = match self.tag_table.lookup(&rgb_str) {
            Some(tag) => tag,
            None => {
                let tag = TextTag::new(Some(&rgb_str));
                tag.set_properties(&[
                    ("foreground", &rgb_str),
                    ("foreground-set", &true),
                    ("weight", &600),
                    ("weight-set", &true)
                ]).unwrap();
                self.tag_table.add(&tag);
                tag
            }
        };

        // Username
        self.buffer.insert_with_tag(
            &mut self.buffer.get_end_iter(),
            from, &color_tag
        );

        // Colon and space after username
        self.buffer.insert(
            &mut self.buffer.get_end_iter(),
            ": "
        );

        // Message
        for part in msg_parts {
            match part {
                MessagePart::Text(text) => self.buffer.insert(
                    &mut self.buffer.get_end_iter(),
                    &text
                ),
                MessagePart::Pixbuf(pixbuf) => self.buffer.insert_pixbuf(
                    &mut self.buffer.get_end_iter(),
                    &pixbuf
                )
            }
        }

        // New line after each message
        self.buffer.insert(
            &mut self.buffer.get_end_iter(),
            "\n\n"
        );

    }

    pub fn append_notice(&self, notice: &str) {

        self.buffer.insert(
            &mut self.buffer.get_end_iter(),
            notice
        );

        self.buffer.insert(
            &mut self.buffer.get_end_iter(),
            "\n\n"
        );

    }

    pub fn clear_chat(&self) {

        self.buffer.set_text("");

    }

    pub fn connect(&self, channel: String) {

        self.channel.replace(channel.clone());
        self.clear_chat();
        let tx = self.buffer_tx.clone();

        rt::run_cb_local(async move {

            let logged_in_user = p!((*USER.lock().unwrap()).clone().ok_or("Not logged in"));

            let user_config = p!(UserConfig::builder()
                .enable_all_capabilities()
                .name(logged_in_user.username.to_lowercase())
                .token(&format!("oauth:{}", logged_in_user.oauth_token))
                .build());

            // We connect to Twitch twice here. Why? Since we don't get our message sent back
            // to us on the same connection, we need to have a second connection so that we
            // see our own message. We COULD use only one connection but that would require
            // parsing the message ourself, figuring out what emotes we have access to, what
            // color our name is and so on. We want Twitch to do all that work so we just open
            // a second connection. This is what seems to be the recomended way.

            // Writer
            let connector_1 = p!(ConnectorNativeTls::twitch());
            let mut runner_1 = p!(AsyncRunner::connect(connector_1, &user_config).await);
            p!(runner_1.join(&channel.to_lowercase()).await);

            // Reader
            let connector_2 = p!(ConnectorNativeTls::twitch());
            let mut runner_2 = p!(AsyncRunner::connect(connector_2, &user_config).await);
            p!(runner_2.join(&channel.to_lowercase()).await);

            let ret = (runner_1.writer(), (runner_1.quit_handle(), runner_2.quit_handle()));

            // Loop to make sure pings and pongs are sent
            tokio::spawn(async move {
                loop {
                    match runner_1.next_message().await {
                        Ok(status) => match status {
                            Status::Message(_) => { },
                            Status::Eof | Status::Quit => break
                        },
                        Err(e) => {
                            warning!("{:#?}", e);
                            break
                        }
                    }
                }
                debug!("Stopping writer (runner_1)");
            });

            tokio::spawn(async move {
                loop {
                    match runner_2.next_message().await {
                        Ok(status) => match status {
                            Status::Message(message) => tx.send(message).expect("RX dropped"),
                            Status::Eof | Status::Quit => break
                        }
                        Err(e) => {
                            warning!("{:#?}", e);
                            break
                        }
                    }
                }
                debug!("Stopping reader (runner_2)");
            });

            Ok(ret)

        }, clone!(
            @strong self.writer as writer,
            @strong self.quit_handle as quit_handle
        => move |msg: GtResult<_>| {
            match msg {
                Ok((w, (q1, q2))) => {
                    writer.borrow_mut().replace(w);
                    quit_handle.borrow_mut().replace((q1, q2));
                },
                Err(e) => {
                    warning!("{}", e);
                    show_info_bar(
                        "Chat error",
                        &e.to_string(),
                        None::<&gtk::Widget>,
                        gtk::MessageType::Error
                    );
                }
            }
        }));

    }

    pub fn disconnect(&self) {

        drop(self.writer.borrow_mut().take());
        self.quit_handle
            .borrow_mut()
            .take()
            .map(|(h1, h2)| rt::run(async move {
                h1.notify().await;
                h2.notify().await;
            }));

    }

    fn scroll_bottom(&self) {

        let scroll_to_bottom = match self.scroll.get_vadjustment() {
            Some(adj) => {
                let height = self.scroll.get_allocated_height() as f64;
                let value = adj.get_value();
                let upper = adj.get_upper();
                match value >= upper - 300f64 - height || value == 0f64 {
                    true => Some(adj),
                    false => None
                }
            },
            None => None
        };

        if let Some(adj) = &scroll_to_bottom {
            adj.set_value(adj.get_upper() + 1000f64)
        }

    }

}

// Whis isn't this in GTK-RS? :(
trait InsertWithTag {
    fn insert_with_tag<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tag: &T) {
        self.insert_with_tags(iter, text, &[tag])
    }
    fn insert_with_tags<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tags: &[&T]);
}

impl InsertWithTag for TextBuffer {
    fn insert_with_tags<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tags: &[&T]) {
        self.insert(iter, text);
        let end_iter = self.get_end_iter();
        iter.backward_chars(text.len() as i32);
        for tag in tags {
            self.apply_tag(*tag, &iter, &end_iter);
        }
    }
}

fn get_color(username: &str) -> RGB {

    let mut index = 0usize;
    for c in username.chars() {
        index += (c as u8) as usize
    }

    DEFAULT_CHAT_COLORS[index % DEFAULT_CHAT_COLORS.len()]

}

// TODO:
// FIXME: Handle UTF-8 stuff.
// https://crates.io/crates/unicode-segmentation
// str::chars may also work
fn partify(msg: String, emotes: Vec<((usize, usize), Pixbuf)>) -> Vec<MessagePart> {

    let mut parts = Vec::new();

    if emotes.len() == 0 {
        parts.push(MessagePart::Text(msg));
        return parts
    }

    for i in 0..emotes.len() {

        if i == 0 && !msg.as_str()[0..emotes[i].0.0].is_empty() {
            parts.push(MessagePart::Text(msg.as_str()[0..emotes[i].0.0].to_string()));
        }

        parts.push(MessagePart::Pixbuf(emotes[i].1.clone()));

        if i + 1 < emotes.len() {
            parts.push(MessagePart::Text(msg.as_str()[emotes[i].0.1+1..emotes[i+1].0.0].to_string()));
        } else if !&msg.as_str()[emotes[i].0.1+1..].is_empty() {
            parts.push(MessagePart::Text(msg.as_str()[emotes[i].0.1+1..].to_string()));
        }

    }

    parts

}
