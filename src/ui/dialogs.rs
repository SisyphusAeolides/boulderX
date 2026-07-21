//! Modal dialogs for Matrix login/join and IRC/MX account management.
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;

use crate::app::{AppInput, AppModel};

/// Discord bot-only login form. Discord user tokens and selfbots are not supported.
pub fn show_discord_login_dialog(
    parent: &gtk::Window,
    sender: &ComponentSender<AppModel>,
    bot_token: &str,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Connect Discord Bot")
        .default_width(440)
        .default_height(250)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_all(18);
    vbox.append(
        &gtk::Label::builder()
            .label("Connect an authorized Discord bot")
            .halign(gtk::Align::Start)
            .css_classes(["dialog-title"])
            .build(),
    );
    vbox.append(&gtk::Label::builder()
        .label("Use a bot token from the Discord Developer Portal. Discord user tokens and selfbots are not supported.")
        .wrap(true).halign(gtk::Align::Start).build());
    let token_entry = gtk::Entry::builder()
        .placeholder_text("Bot token")
        .text(bot_token)
        .visibility(false)
        .build();
    vbox.append(&gtk::Label::new(Some("Bot token:")));
    vbox.append(&token_entry);
    let remember = gtk::CheckButton::builder()
        .label("Remember on this device")
        .active(true)
        .build();
    vbox.append(&remember);
    let status = gtk::Label::builder().halign(gtk::Align::Start).build();
    vbox.append(&status);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    buttons.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancel");
    let close = dialog.clone();
    cancel.connect_clicked(move |_| close.close());
    buttons.append(&cancel);
    let connect = gtk::Button::with_label("Connect");
    connect.add_css_class("suggested-action");
    let s = sender.clone();
    let close = dialog.clone();
    let entry = token_entry.clone();
    let remember_box = remember.clone();
    let status_label = status.clone();
    connect.connect_clicked(move |_| {
        let bot_token = entry.text().trim().to_string();
        if bot_token.is_empty() {
            status_label.set_label("A bot token is required.");
            return;
        }
        s.input(AppInput::DiscordLogin {
            bot_token,
            remember: remember_box.is_active(),
        });
        close.close();
    });
    buttons.append(&connect);
    vbox.append(&buttons);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

pub fn show_matrix_login_dialog(
    parent: &gtk::Window,
    sender: &ComponentSender<AppModel>,
    homeserver: &str,
    username: &str,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Sign in to Matrix")
        .default_width(420)
        .default_height(340)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.set_margin_all(20);
    let title = gtk::Label::builder()
        .label("Matrix Login")
        .halign(gtk::Align::Start)
        .build();
    title.add_css_class("dialog-title");
    vbox.append(&title);

    let hs_default = if homeserver.is_empty() {
        "https://matrix.org"
    } else {
        homeserver
    };
    let hs_entry = gtk::Entry::builder()
        .placeholder_text("Homeserver (e.g. https://matrix.org)")
        .text(hs_default)
        .build();
    vbox.append(&gtk::Label::new(Some("Homeserver:")));
    vbox.append(&hs_entry);

    let user_entry = gtk::Entry::builder()
        .placeholder_text("@user:matrix.org")
        .text(username)
        .build();
    vbox.append(&gtk::Label::new(Some("Username:")));
    vbox.append(&user_entry);

    let pass_entry = gtk::Entry::builder()
        .placeholder_text("Password")
        .visibility(false)
        .build();
    vbox.append(&gtk::Label::new(Some("Password:")));
    vbox.append(&pass_entry);

    let remember = gtk::CheckButton::builder()
        .label("Remember on this device")
        .active(true)
        .build();
    vbox.append(&remember);

    let status = gtk::Label::builder()
        .label("")
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    status.add_css_class("status-connecting");
    vbox.append(&status);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);
    let cancel = gtk::Button::with_label("Cancel");
    let d1 = dialog.clone();
    cancel.connect_clicked(move |_| d1.close());
    btn_box.append(&cancel);

    let login_btn = gtk::Button::with_label("Sign In");
    login_btn.add_css_class("suggested-action");
    let s = sender.clone();
    let d2 = dialog.clone();
    let hs = hs_entry.clone();
    let usr = user_entry.clone();
    let pwd = pass_entry.clone();
    let st = status.clone();
    let rem = remember.clone();
    login_btn.connect_clicked(move |_| {
        let homeserver = hs.text().to_string().trim().to_string();
        let username = usr.text().to_string().trim().to_string();
        let password = pwd.text().to_string();
        if homeserver.is_empty() || username.is_empty() || password.is_empty() {
            st.set_label("All fields required.");
            return;
        }
        st.set_label("Connecting\u{2026}");
        s.input(AppInput::MatrixLogin {
            homeserver,
            username,
            password,
            remember: rem.is_active(),
        });
        d2.close();
    });
    btn_box.append(&login_btn);
    vbox.append(&btn_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

pub fn show_matrix_join_dialog(parent: &gtk::Window, sender: &ComponentSender<AppModel>) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Join Matrix Room")
        .default_width(380)
        .default_height(180)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    let entry = gtk::Entry::builder()
        .placeholder_text("#room:matrix.org or !roomid:server")
        .build();
    vbox.append(&gtk::Label::new(Some("Room alias or ID:")));
    vbox.append(&entry);
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancel");
    let d1 = dialog.clone();
    cancel.connect_clicked(move |_| d1.close());
    btn_box.append(&cancel);
    let join_btn = gtk::Button::with_label("Join");
    join_btn.add_css_class("suggested-action");
    let s = sender.clone();
    let d2 = dialog.clone();
    let e = entry.clone();
    join_btn.connect_clicked(move |_| {
        let alias = e.text().to_string().trim().to_string();
        if !alias.is_empty() {
            s.input(AppInput::MatrixJoinRoom(alias));
            d2.close();
        }
    });
    btn_box.append(&join_btn);
    vbox.append(&btn_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

/// IRC login / connect form — nick, server, NickServ/SASL password.
pub fn show_irc_login_dialog(
    parent: &gtk::Window,
    sender: &ComponentSender<AppModel>,
    server: &str,
    nick: &str,
    password: &str,
    auth_method: &str,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("IRC Login")
        .default_width(440)
        .default_height(380)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_all(18);
    let title = gtk::Label::builder()
        .label("Connect to IRC")
        .halign(gtk::Align::Start)
        .build();
    title.add_css_class("dialog-title");
    vbox.append(&title);

    let server_entry = gtk::Entry::builder()
        .placeholder_text("irc.libera.chat")
        .text(server)
        .build();
    vbox.append(&gtk::Label::new(Some("Server:")));
    vbox.append(&server_entry);

    let nick_entry = gtk::Entry::builder()
        .placeholder_text("Nickname")
        .text(nick)
        .build();
    vbox.append(&gtk::Label::new(Some("Nickname:")));
    vbox.append(&nick_entry);

    let pass_entry = gtk::Entry::builder()
        .placeholder_text("NickServ / SASL password (optional)")
        .text(password)
        .visibility(false)
        .build();
    vbox.append(&gtk::Label::new(Some("Password:")));
    vbox.append(&pass_entry);

    let auth_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    auth_row.append(&gtk::Label::new(Some("Auth:")));
    let auth_combo =
        gtk::DropDown::from_strings(&["nickserv", "sasl_plain", "sasl_external", "none"]);
    let auth_idx = match auth_method {
        "sasl_plain" => 1_u32,
        "sasl_external" => 2,
        "none" => 3,
        _ => 0,
    };
    auth_combo.set_selected(auth_idx);
    auth_row.append(&auth_combo);
    vbox.append(&auth_row);

    let status = gtk::Label::builder()
        .label("Libera.Chat and most networks need NickServ or SASL for registered nicks.")
        .wrap(true)
        .halign(gtk::Align::Start)
        .build();
    vbox.append(&status);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);

    let cancel = gtk::Button::with_label("Cancel");
    let d0 = dialog.clone();
    cancel.connect_clicked(move |_| d0.close());
    btn_box.append(&cancel);

    let connect_btn = gtk::Button::with_label("Connect");
    connect_btn.add_css_class("suggested-action");
    let s = sender.clone();
    let d1 = dialog.clone();
    let se = server_entry.clone();
    let ne = nick_entry.clone();
    let pe = pass_entry.clone();
    let ac = auth_combo.clone();
    let st = status.clone();
    connect_btn.connect_clicked(move |_| {
        let srv = se.text().to_string().trim().to_string();
        let nick = ne.text().to_string().trim().to_string();
        let password = pe.text().to_string();
        if srv.is_empty() || nick.is_empty() {
            st.set_label("Server and nickname are required.");
            return;
        }
        let auth = match ac.selected() {
            1 => "sasl_plain",
            2 => "sasl_external",
            3 => "none",
            _ => "nickserv",
        }
        .to_string();
        s.input(AppInput::UpdateServer(srv));
        s.input(AppInput::UpdateNickname(nick));
        s.input(AppInput::UpdatePassword(password));
        s.input(AppInput::UpdateAuthMethod(auth));
        s.input(AppInput::SaveSettings);
        s.input(AppInput::Connect);
        d1.close();
    });
    btn_box.append(&connect_btn);
    vbox.append(&btn_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

pub fn show_register_dialog(parent: &gtk::Window, sender: &ComponentSender<AppModel>, nick: &str) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Register IRC Nickname")
        .default_width(400)
        .default_height(280)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    vbox.append(&gtk::Label::new(Some(
        "Registers with NickServ after connect (Libera.Chat style).",
    )));
    let nick_entry = gtk::Entry::builder().text(nick).build();
    vbox.append(&gtk::Label::new(Some("Nickname:")));
    vbox.append(&nick_entry);
    let pass_entry = gtk::Entry::builder()
        .visibility(false)
        .placeholder_text("Password")
        .build();
    vbox.append(&gtk::Label::new(Some("Password:")));
    vbox.append(&pass_entry);
    let email_entry = gtk::Entry::builder()
        .placeholder_text("Email (optional, network-dependent)")
        .build();
    vbox.append(&gtk::Label::new(Some("Email:")));
    vbox.append(&email_entry);
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancel");
    let d0 = dialog.clone();
    cancel.connect_clicked(move |_| d0.close());
    btn_box.append(&cancel);
    let ok = gtk::Button::with_label("Register");
    ok.add_css_class("suggested-action");
    let s = sender.clone();
    let d1 = dialog.clone();
    let n = nick_entry.clone();
    let p = pass_entry.clone();
    let e = email_entry.clone();
    ok.connect_clicked(move |_| {
        let nick = n.text().to_string().trim().to_string();
        let password = p.text().to_string();
        let email = e.text().to_string().trim().to_string();
        if nick.is_empty() || password.is_empty() {
            return;
        }
        s.input(AppInput::SubmitRegistration {
            nick,
            password,
            email,
        });
        d1.close();
    });
    btn_box.append(&ok);
    vbox.append(&btn_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

pub fn show_verify_dialog(parent: &gtk::Window, sender: &ComponentSender<AppModel>, nick: &str) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Verify IRC Registration")
        .default_width(380)
        .default_height(200)
        .build();
    dialog.add_css_class("boulder-relay");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_all(16);
    let nick_entry = gtk::Entry::builder().text(nick).build();
    vbox.append(&gtk::Label::new(Some("Nickname:")));
    vbox.append(&nick_entry);
    let code_entry = gtk::Entry::builder()
        .placeholder_text("Verification code from email")
        .build();
    vbox.append(&gtk::Label::new(Some("Code:")));
    vbox.append(&code_entry);
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancel");
    let d0 = dialog.clone();
    cancel.connect_clicked(move |_| d0.close());
    btn_box.append(&cancel);
    let ok = gtk::Button::with_label("Verify");
    ok.add_css_class("suggested-action");
    let s = sender.clone();
    let d1 = dialog.clone();
    let n = nick_entry.clone();
    let c = code_entry.clone();
    ok.connect_clicked(move |_| {
        let nick = n.text().to_string().trim().to_string();
        let code = c.text().to_string().trim().to_string();
        if nick.is_empty() || code.is_empty() {
            return;
        }
        s.input(AppInput::SubmitVerification { nick, code });
        d1.close();
    });
    btn_box.append(&ok);
    vbox.append(&btn_box);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

/// Combined IRC + Matrix account manager.
pub fn show_account_manager(
    parent: &gtk::Window,
    sender: &ComponentSender<AppModel>,
    irc_server: &str,
    irc_nick: &str,
    irc_password: &str,
    irc_auth: &str,
    account_service: &str,
    matrix_homeserver: &str,
    matrix_user: &str,
    discord_bot_token: &str,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Account Manager")
        .default_width(520)
        .default_height(480)
        .build();
    dialog.add_css_class("boulder-relay");

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    let switcher = gtk::StackSwitcher::new();
    switcher.set_stack(Some(&stack));
    switcher.set_halign(gtk::Align::Center);
    switcher.set_margin_top(8);
    switcher.set_margin_bottom(4);
    outer.append(&switcher);

    // ── IRC page ──
    let irc_page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    irc_page.set_margin_all(16);
    irc_page.append(
        &gtk::Label::builder()
            .label("IRC account (per-server)")
            .halign(gtk::Align::Start)
            .css_classes(["dialog-title"])
            .build(),
    );

    let srv_entry = gtk::Entry::builder().text(irc_server).build();
    irc_page.append(&gtk::Label::new(Some("Server:")));
    irc_page.append(&srv_entry);

    let nick_entry = gtk::Entry::builder().text(irc_nick).build();
    irc_page.append(&gtk::Label::new(Some("Nickname:")));
    irc_page.append(&nick_entry);

    let pass_entry = gtk::Entry::builder()
        .text(irc_password)
        .visibility(false)
        .placeholder_text("NickServ / SASL password")
        .build();
    irc_page.append(&gtk::Label::new(Some("Password:")));
    irc_page.append(&pass_entry);

    let svc_entry = gtk::Entry::builder().text(account_service).build();
    irc_page.append(&gtk::Label::new(Some(
        "Services bot (NickServ / AuthServ):",
    )));
    irc_page.append(&svc_entry);

    let auth_combo =
        gtk::DropDown::from_strings(&["nickserv", "sasl_plain", "sasl_external", "none"]);
    auth_combo.set_selected(match irc_auth {
        "sasl_plain" => 1,
        "sasl_external" => 2,
        "none" => 3,
        _ => 0,
    });
    irc_page.append(&gtk::Label::new(Some("Auth method:")));
    irc_page.append(&auth_combo);

    let irc_btns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    irc_btns.set_halign(gtk::Align::End);
    let save_irc = gtk::Button::with_label("Save IRC");
    save_irc.add_css_class("suggested-action");
    let s = sender.clone();
    let se = srv_entry.clone();
    let ne = nick_entry.clone();
    let pe = pass_entry.clone();
    let sve = svc_entry.clone();
    let ac = auth_combo.clone();
    save_irc.connect_clicked(move |_| {
        let auth = match ac.selected() {
            1 => "sasl_plain",
            2 => "sasl_external",
            3 => "none",
            _ => "nickserv",
        }
        .to_string();
        s.input(AppInput::UpdateServer(se.text().trim().to_string()));
        s.input(AppInput::UpdateNickname(ne.text().trim().to_string()));
        s.input(AppInput::UpdatePassword(pe.text().to_string()));
        s.input(AppInput::UpdateAccountService(
            sve.text().trim().to_string(),
        ));
        s.input(AppInput::UpdateAuthMethod(auth));
        s.input(AppInput::SaveSettings);
    });
    irc_btns.append(&save_irc);

    let reg = gtk::Button::with_label("Register…");
    let s2 = sender.clone();
    let d_reg = dialog.clone();
    let n_reg = nick_entry.clone();
    reg.connect_clicked(move |_| {
        show_register_dialog(&d_reg, &s2, &n_reg.text());
    });
    irc_btns.append(&reg);

    let ver = gtk::Button::with_label("Verify…");
    let s3 = sender.clone();
    let d_ver = dialog.clone();
    let n_ver = nick_entry.clone();
    ver.connect_clicked(move |_| {
        show_verify_dialog(&d_ver, &s3, &n_ver.text());
    });
    irc_btns.append(&ver);

    let login_irc = gtk::Button::with_label("Connect IRC");
    let s4 = sender.clone();
    let d_login = dialog.clone();
    let se2 = srv_entry.clone();
    let ne2 = nick_entry.clone();
    let pe2 = pass_entry.clone();
    let ac2 = auth_combo.clone();
    login_irc.connect_clicked(move |_| {
        let auth = match ac2.selected() {
            1 => "sasl_plain",
            2 => "sasl_external",
            3 => "none",
            _ => "nickserv",
        }
        .to_string();
        s4.input(AppInput::UpdateServer(se2.text().trim().to_string()));
        s4.input(AppInput::UpdateNickname(ne2.text().trim().to_string()));
        s4.input(AppInput::UpdatePassword(pe2.text().to_string()));
        s4.input(AppInput::UpdateAuthMethod(auth));
        s4.input(AppInput::SaveSettings);
        s4.input(AppInput::Connect);
        d_login.close();
    });
    irc_btns.append(&login_irc);
    irc_page.append(&irc_btns);
    stack.add_titled(&irc_page, Some("irc"), "IRC");

    // ── Matrix page ──
    let mx_page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    mx_page.set_margin_all(16);
    mx_page.append(
        &gtk::Label::builder()
            .label("Matrix account")
            .halign(gtk::Align::Start)
            .css_classes(["dialog-title"])
            .build(),
    );

    let mx_hs = gtk::Entry::builder()
        .text(if matrix_homeserver.is_empty() {
            "https://matrix.org"
        } else {
            matrix_homeserver
        })
        .build();
    mx_page.append(&gtk::Label::new(Some("Homeserver:")));
    mx_page.append(&mx_hs);

    let mx_user = gtk::Entry::builder()
        .text(matrix_user)
        .placeholder_text("@user:matrix.org")
        .build();
    mx_page.append(&gtk::Label::new(Some("Username:")));
    mx_page.append(&mx_user);

    let mx_pass = gtk::Entry::builder()
        .visibility(false)
        .placeholder_text("Password")
        .build();
    mx_page.append(&gtk::Label::new(Some("Password:")));
    mx_page.append(&mx_pass);

    let mx_btns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    mx_btns.set_halign(gtk::Align::End);
    let clear_mx = gtk::Button::with_label("Clear saved");
    let s_clear = sender.clone();
    clear_mx.connect_clicked(move |_| {
        s_clear.input(AppInput::ClearMatrixAccount);
    });
    mx_btns.append(&clear_mx);

    let sign_mx = gtk::Button::with_label("Sign In");
    sign_mx.add_css_class("suggested-action");
    let s_mx = sender.clone();
    let d_mx = dialog.clone();
    let hs = mx_hs.clone();
    let us = mx_user.clone();
    let pw = mx_pass.clone();
    sign_mx.connect_clicked(move |_| {
        let homeserver = hs.text().trim().to_string();
        let username = us.text().trim().to_string();
        let password = pw.text().to_string();
        if homeserver.is_empty() || username.is_empty() || password.is_empty() {
            return;
        }
        s_mx.input(AppInput::MatrixLogin {
            homeserver,
            username,
            password,
            remember: true,
        });
        d_mx.close();
    });
    mx_btns.append(&sign_mx);
    mx_page.append(&mx_btns);
    stack.add_titled(&mx_page, Some("matrix"), "Matrix");

    // ── Discord page ──
    let discord_page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    discord_page.set_margin_all(16);
    discord_page.append(
        &gtk::Label::builder()
            .label("Discord bot account")
            .halign(gtk::Align::Start)
            .css_classes(["dialog-title"])
            .build(),
    );
    discord_page.append(&gtk::Label::builder()
        .label("Only authorized Discord bot tokens are supported. User tokens and selfbots are not supported.")
        .halign(gtk::Align::Start).wrap(true).build());
    let discord_token = gtk::Entry::builder()
        .text(discord_bot_token)
        .visibility(false)
        .placeholder_text("Bot token")
        .build();
    discord_page.append(&gtk::Label::new(Some("Bot token:")));
    discord_page.append(&discord_token);
    let discord_btns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    discord_btns.set_halign(gtk::Align::End);
    let clear_discord = gtk::Button::with_label("Clear saved");
    let s_clear_discord = sender.clone();
    clear_discord.connect_clicked(move |_| s_clear_discord.input(AppInput::ClearDiscordAccount));
    discord_btns.append(&clear_discord);
    let connect_discord = gtk::Button::with_label("Connect Discord");
    connect_discord.add_css_class("suggested-action");
    let s_discord = sender.clone();
    let d_discord = dialog.clone();
    let token = discord_token.clone();
    connect_discord.connect_clicked(move |_| {
        let bot_token = token.text().trim().to_string();
        if !bot_token.is_empty() {
            s_discord.input(AppInput::DiscordLogin {
                bot_token,
                remember: true,
            });
            d_discord.close();
        }
    });
    discord_btns.append(&connect_discord);
    discord_page.append(&discord_btns);
    stack.add_titled(&discord_page, Some("discord"), "Discord");

    outer.append(&stack);

    let close_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    close_row.set_halign(gtk::Align::End);
    close_row.set_margin_all(12);
    let close = gtk::Button::with_label("Close");
    let d_close = dialog.clone();
    close.connect_clicked(move |_| d_close.close());
    close_row.append(&close);
    outer.append(&close_row);

    dialog.set_child(Some(&outer));
    dialog.present();
}
