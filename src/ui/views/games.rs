use {
    crate::{
        USER, get_obj, rt, warning,
        twitch::{Twitch, TwitchUtils, response::Stream},
        resources::{CLIENT_ID, REQUEST_SIZE, GAME_COVER_SIZE},
        ui::show_info_bar
    },
    super::super::cards::GameCard,
    std::{rc::Rc, cell::RefCell},
    gtk::{Builder, FlowBox, ScrolledWindow, prelude::*},
    glib::{clone, Sender}
};

// TODO: Handle errors

pub struct GamesView {
    flow: Rc<FlowBox>,
    scroll: ScrolledWindow,
    pagination: Rc<RefCell<Option<String>>>,
    tx: Sender<Stream>
}

impl GamesView {

    pub fn configure(builder: &Builder, tx: Sender<Stream>) -> Rc<Self> {

        let inner = Rc::new(Self {
            flow: Rc::new(get_obj!(builder, "games-flowbox")),
            scroll: get_obj!(builder, "games-scroll-window"),
            pagination: Rc::new(RefCell::new(None)),
            tx
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

            let token = user.oauth_token.clone();
            let flow = Rc::clone(&self.flow);
            let pagination = Rc::clone(&self.pagination);
            let pagi_str = self.pagination.borrow().clone();

            rt::run_cb_local(
                async move {
                    let tw = Twitch::new(Some(token), Some(CLIENT_ID.into()));
                    tw.get_top_games(pagi_str, None, Some(REQUEST_SIZE)).await
                },
                clone!(@strong flow, @strong pagination => move |res| {
                    match res {
                        Ok(tw_response) => {
                            if let Some(pagi) = tw_response.pagination {
                                if let Some(cursor) = pagi.cursor {
                                    pagination.borrow_mut().replace(cursor);
                                }
                            }
                            for game in tw_response.data {
                                let card = GameCard::new(
                                    TwitchUtils::thumbnail_sizer(&game.box_art_url, GAME_COVER_SIZE.0, GAME_COVER_SIZE.1),
                                    &game.name
                                );
                                flow.insert(card.get_widget(), -1);
                            }
                        },
                        Err(e) => {
                            warning!("{}", e);
                            show_info_bar(
                                "Error loading channels",
                                &e.to_string(),
                                None::<&gtk::Widget>,
                                gtk::MessageType::Error
                            );
                        }
                    }
                })
            );

        }

    }

    pub fn refresh(&self) {
        *self.pagination.borrow_mut() = None;
        self.clear();
        self.load_next()
    }

    pub fn clear(&self) {
        for c in self.flow.get_children() {
            self.flow.remove(&c);
        }
    }

}
