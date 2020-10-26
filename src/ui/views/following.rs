use {
    crate::{
        USER, get_obj, rt, warning,
        resources::{STREAM_COVER_SIZE, CLIENT_ID},
        twitch::{Twitch, TwitchUtils, TwResult, response::Stream}
    },
    super::super::cards::LiveCard,
    std::rc::Rc,
    gtk::{Builder, FlowBox, ScrolledWindow, prelude::*},
    glib::{clone, Sender}
};

// TODO: Handle errors

pub struct FollowingView {
    flow: Rc<FlowBox>,
    scroll: ScrolledWindow,
    tx: Sender<(String, String)>
}

impl FollowingView {

    pub fn configure(builder: &Builder, tx: Sender<(String, String)>) -> Rc<Self> {

        let inner = Rc::new(Self {
            flow: Rc::new(get_obj!(builder, "following-flowbox")),
            scroll: get_obj!(builder, "following-scroll-window"),
            tx
        });

        inner.flow.connect_child_activated(|_, child| {
            // child.notify("button-press-event");
            child.event(&gdk::Event::new(gdk::EventType::ButtonPress));

        });

        inner.scroll.connect_edge_reached(clone!(@strong inner => move |_, pos| {
            if pos == gtk::PositionType::Bottom {
                inner.load_next()
            }
        }));

        inner

    }

    pub fn load_next(&self) {

        if let Some(ref user) = *USER.lock().unwrap() {

            let user_id = user.user_id.clone();
            let token = user.oauth_token.clone();
            let flow = Rc::clone(&self.flow);
            let tx = self.tx.clone();

            rt::run_cb_local(async move {

                let tw = Twitch::new(Some(token), Some(CLIENT_ID.into()));

                let res: TwResult<Vec<Stream>> = async {

                    let mut following_ids = Vec::<String>::new();
                    let mut streams = Vec::<Stream>::new();

                    let mut pagination: Option<String> = None;
                    loop {
                        let following = tw.get_users_follows(pagination.clone(), Some(100), Some(user_id.clone()), None).await?;
                        for f in following.data {
                            following_ids.push(f.to_id);
                        }
                        match following.pagination {
                            Some(pagi) => match pagi.cursor {
                                Some(cursor) =>{
                                    pagination = Some(cursor);
                                    continue
                                },
                                None => break
                            },
                            None => break
                        }
                    }

                    for i in 0..((following_ids.len() as f64 / 100f64).ceil() as usize) {
                        let s = i * 100;
                        let e = (i * 100) + 100;
                        let temp_streams = if e > following_ids.len() {
                            tw.get_streams(None, None, Some(100), None, None, Some(following_ids[s..].to_vec()), None).await?
                        } else {
                            tw.get_streams(None, None, Some(100), None, None, Some(following_ids[s..e].to_vec()), None).await?
                        };
                        for stream in temp_streams.data {
                            streams.push(stream);
                        }
                    }

                    Ok(streams)

                }.await;

                res

            }, clone!(@strong flow => move |res| {
                match res {
                    Ok(streams) => for stream in streams {
                        let lc = LiveCard::new(
                            TwitchUtils::thumbnail_sizer(&stream.thumbnail_url, STREAM_COVER_SIZE.0, STREAM_COVER_SIZE.1),
                            stream.title,
                            stream.user_name,
                            tx.clone()
                        );
                        flow.insert(lc.get_widget(), -1);
                    },
                    Err(e) => warning!("{}", e)
                }
            }));

        }

    }

    pub fn refresh(&self) {
        self.clear();
        self.load_next()
    }

    pub fn clear(&self) {
        for c in self.flow.get_children() {
            self.flow.remove(&c);
        }
    }

}
