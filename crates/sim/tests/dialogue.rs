//! Conversations, through `World::step`: Interact on someone in reach begins one and goes
//! through its lines; the view turns to the speaker; choices come after a node's last
//! line and are taken by number; a node sets flags and gives Marks; the next talk is
//! chosen by the flags; a Light character says one line at a time in turn; a witness
//! line is said only with two players; and the flags are saved.

use glam::Vec3;
use vellmark_content::{Choice, Condition, DialogueTable, Line, Node, Speaker, Talk};
use vellmark_sim::dialogue::Talker;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig};

fn line(text: u64) -> Line {
    Line { by: "wick".into(), text, when: vec![] }
}

fn table() -> DialogueTable {
    DialogueTable {
        speakers: vec![
            Speaker {
                key: "wick".into(),
                talks: vec![Talk { when: vec![Condition::Not("spoke".into())], start: 0 }, Talk { when: vec![Condition::Flag("spoke".into())], start: 3 }],
                nodes: vec![
                    Node {
                        lines: vec![line(1), line(2), Line { by: "wick".into(), text: 99, when: vec![Condition::Witness] }],
                        choices: vec![
                            Choice { text: 10, when: vec![], then: Some(1), set: vec!["asked_where".into()] },
                            Choice { text: 11, when: vec![], then: Some(2), set: vec![] },
                            Choice { text: 12, when: vec![Condition::Flag("never".into())], then: None, set: vec![] },
                        ],
                        ..Default::default()
                    },
                    Node { lines: vec![line(3)], next: Some(2), ..Default::default() },
                    Node { lines: vec![line(4)], set: vec!["spoke".into()], marks: 12, ..Default::default() },
                    Node { lines: vec![line(5)], ..Default::default() },
                ],
                light: vec![],
            },
            Speaker { key: "villager".into(), talks: vec![], nodes: vec![], light: vec![20, 21] },
        ],
    }
}

fn world(players: usize) -> World {
    let mut w = World::new(WorldConfig::default());
    for k in 0..players {
        w.add_player(Vec3::new(k as f32 * 0.6, 0.0, 0.0), 0.0);
    }
    w.set_dialogue(table());
    // Wick, a little to the right and ahead; the villager behind.
    w.add_talker(Talker { id: 7, speaker: "wick".into(), feet: Vec3::new(0.4, 0.0, -2.0), height: 1.75 });
    w.add_talker(Talker { id: 8, speaker: "villager".into(), feet: Vec3::new(0.0, 0.0, 2.0), height: 1.7 });
    w
}

fn press(w: &mut World, input: PlayerInput) {
    let n = w.snapshot().players.len();
    let mut inputs = vec![PlayerInput::default(); n];
    inputs[0] = input;
    w.step(&inputs);
    w.step(&vec![PlayerInput::default(); n]);
}

fn interact(w: &mut World) {
    press(w, PlayerInput { buttons: buttons::INTERACT, ..Default::default() });
}

fn said(w: &World) -> Option<u64> {
    w.snapshot().players[0].talking.as_ref().and_then(|t| t.line)
}

#[test]
fn a_conversation_goes_through_its_lines_and_choices() {
    let mut w = world(1);
    // Wick is near enough the gaze to be spoken to.
    assert_eq!(w.snapshot().players[0].talker_in_gaze.as_deref(), Some("wick"));
    interact(&mut w);
    assert_eq!(said(&w), Some(1));
    // The view turns to his face.
    for _ in 0..30 {
        w.step(&[PlayerInput { move_y: 1.0, look_yaw: -0.2, ..Default::default() }]);
    }
    let p = &w.snapshot().players[0];
    assert!(p.yaw > 0.1 && p.pitch > -0.05, "facing him: yaw {} pitch {}", p.yaw, p.pitch);
    assert!(Vec3::new(p.feet.x, 0.0, p.feet.z).length() < 0.05, "no walking off while he speaks: {:?}", p.feet);
    interact(&mut w);
    assert_eq!(said(&w), Some(2));
    // The witness line is skipped with one player; the choices come, the one whose flag
    // is not set left out.
    interact(&mut w);
    let t = w.snapshot().players[0].talking.clone().unwrap();
    assert_eq!(t.choices, vec![10, 11]);
    assert_eq!(t.line, Some(2), "the last line stays above the choices");
    // Interact alone does not choose.
    interact(&mut w);
    assert_eq!(w.snapshot().players[0].talking.as_ref().unwrap().choices.len(), 2);
    press(&mut w, PlayerInput { say: 1, ..Default::default() });
    assert!(w.flags().contains("asked_where"));
    assert_eq!(said(&w), Some(3));
    interact(&mut w);
    assert_eq!(said(&w), Some(4));
    assert_eq!(w.snapshot().players[0].marks, 12, "the node gave its Marks");
    assert!(w.flags().contains("spoke"));
    interact(&mut w);
    assert!(w.snapshot().players[0].talking.is_none(), "the conversation is over");
    // Spoken to again, he begins the talk the flags now choose.
    interact(&mut w);
    assert_eq!(said(&w), Some(5));
}

#[test]
fn a_light_character_says_one_line_at_a_time_in_turn() {
    let mut w = world(1);
    w.step(&[PlayerInput { look_yaw: std::f32::consts::PI, ..Default::default() }]);
    assert_eq!(w.snapshot().players[0].talker_in_gaze.as_deref(), Some("villager"));
    let mut heard = Vec::new();
    for _ in 0..3 {
        interact(&mut w);
        heard.push(said(&w).unwrap());
        interact(&mut w);
        assert!(w.snapshot().players[0].talking.is_none());
        // Turn back to the villager after each, since the view turned to the face.
        let p = w.snapshot().players[0].clone();
        let want = std::f32::consts::PI;
        w.step(&[PlayerInput { look_yaw: want - p.yaw, look_pitch: -p.pitch, ..Default::default() }]);
    }
    assert_eq!(heard, vec![20, 21, 20]);
}

#[test]
fn a_witness_line_is_said_only_with_two_players_and_flags_are_saved() {
    let mut w = world(2);
    interact(&mut w);
    interact(&mut w);
    interact(&mut w);
    assert_eq!(said(&w), Some(99), "the second player's line");
    interact(&mut w);
    press(&mut w, PlayerInput { say: 2, ..Default::default() });
    assert!(w.flags().contains("spoke"));
    let save = w.save();
    let bytes = save.to_bytes();
    let mut fresh = world(2);
    fresh.load(&vellmark_sim::save::SaveGame::from_bytes(&bytes).unwrap());
    assert!(fresh.flags().contains("spoke"), "the flags came back");
    assert!(fresh.snapshot().players[0].talking.is_none(), "a conversation is not saved part-way");
}
