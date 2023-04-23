use std::collections::HashMap;

use log::{error, debug};
use wasm_bindgen::{JsCast};
use web_sys::{HtmlElement, HtmlDivElement, MouseEvent, HtmlButtonElement, DomRect, window, HtmlSelectElement};

use crate::{dom::{create_element}, model::tournament::{StageId, TournamentId, StageKind, FixtureId, FixtureTeam}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

use super::create_callback_with_arg;

//TODO: drag fixture around

pub struct BracketView {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_root: HtmlElement,
    canvas: HtmlDivElement,
    fixture_divs: HashMap<FixtureId, HtmlDivElement>,

    #[allow(dyn_drop)]
    closures: Vec<Box<dyn Drop>>,

    current_drag: Option<DragInfo>,
}

struct DragInfo {
    fixture_id: FixtureId,
    start_offset: (f64, f64),
}

impl BracketView {
    pub fn get_id(&self) -> UiElementId {
        self.id
    }

    pub fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        if Some(tournament_id) == self.tournament_id {
            self.refresh(model);
        }
    }

    pub fn process_events(&mut self, events: &EventList, model: &Model) {
        for e in events.get_events() {
            match e {
                Event::SelectedTournamentAndStageChanged { source, new_tournament_id, new_stage_id } if *source == self.linked_outline_id => {
                    self.tournament_id = *new_tournament_id;
                    self.stage_id = *new_stage_id;
                    self.refresh(model);
                }
                _ => (),
            }
        }
    }

    pub fn get_dom_root(&self) -> &HtmlElement {
        &self.dom_root
    }

    pub fn new(id: UiElementId, model: &Model, linked_outline_id: UiElementId) -> BracketView {
        let dom_root = create_element::<HtmlElement>("div");
        dom_root.set_inner_text("Bracket view!");

        let canvas = create_element::<HtmlDivElement>("div");
        canvas.set_class_name("bracket-view-canvas");
        canvas.style().set_property("position", "relative").expect("Failed to set property"); // For children to be absolutely positioned relative to this.
        dom_root.append_child(&canvas).expect("Failed to append child");

        #[allow(dyn_drop)]
        let mut closures: Vec<Box<dyn Drop>> = vec![];

        let dblclick_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                this.on_background_dblclick(model, e);
            }
        }));
        canvas.set_ondblclick(Some(dblclick_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(dblclick_closure); // Needs to be kept alive

        let mousemove_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                this.on_background_mousemove(model, e);
            }
        }));
        canvas.set_onmousemove(Some(mousemove_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(mousemove_closure); // Needs to be kept alive

        let mouseup_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                this.on_background_mouseup(model, e);
            }
        }));
        canvas.set_onmouseup(Some(mouseup_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(mouseup_closure); // Needs to be kept alive


        let new_fixture_controls = create_element::<HtmlElement>("div");
        new_fixture_controls.set_inner_text("Double-click to add new fixture with these settings: ");
        let new_fixture_team_a_select = create_element::<HtmlSelectElement>("select");
        // new_fixture_team_a_select.options().add_with_html_option_element(element)
        new_fixture_controls.append_child(&new_fixture_team_a_select).expect("Failed to append child");
        let new_fixture_team_b_select = create_element::<HtmlSelectElement>("select");
        new_fixture_controls.append_child(&new_fixture_team_b_select).expect("Failed to append child");


        dom_root.append_child(&new_fixture_controls).expect("Failed to append child");

        let mut result = BracketView { id, tournament_id: None, stage_id: None, linked_outline_id, dom_root, canvas,
            fixture_divs: HashMap::<FixtureId, HtmlDivElement>::new(), closures,
            current_drag: None };

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        let mut show = false;
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(s) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { .. } = s.kind {
                    show = true;
                }
            }
        }
        self.dom_root.style().set_property("display",
            if show { "block" } else { "none" }).expect("Failed to set style");

        self.canvas.set_inner_html("");
        //TODO: delete closures?

        self.fixture_divs.clear();

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { fixtures } = &stage.kind {
                    for (fid, f) in fixtures {
                        let new_div: HtmlDivElement = create_element::<HtmlDivElement>("div");
                        new_div.set_inner_text(&format!("Fixture {}", fid));
                        new_div.style().set_property("position", "absolute").expect("Failed to set property");
                        new_div.style().set_property("left", &f.layout.0.to_string()).expect("Failed to set property");
                        new_div.style().set_property("top", &f.layout.1.to_string()).expect("Failed to set property");

                        let delete_button: HtmlButtonElement = create_element("button");
                        delete_button.set_inner_text("X");
                        new_div.append_child(&delete_button).expect("Failed to append button");
                        let id = self.id;
                        let fid = *fid;
                        let click_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                                this.on_delete_fixture_button_click(model, fid);
                            }
                        }));
                        delete_button.set_onclick(Some(click_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(click_closure); // Needs to be kept alive

                        let drag_handle: HtmlElement = create_element("span");
                        drag_handle.set_inner_text("::");
                        new_div.append_child(&drag_handle).expect("Failed to append child");
                        let mousedown_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_drag_handle_mousedown(model, fid, e);
                            }
                        }));
                        drag_handle.set_onmousedown(Some(mousedown_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mousedown_closure); // Needs to be kept alive

                        self.canvas.append_child(&new_div).expect("Failed to append child");

                        self.fixture_divs.insert(fid, new_div);
                    }
                }
            }
        }
    }

    fn on_background_dblclick(&self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let None = model.add_fixture(tournament_id, stage_id, (e.offset_x(), e.offset_y()), FixtureTeam::Fixed(0), FixtureTeam::Fixed(0)) {
                error!("Failed to add fixture");
            }
        }
    }

    fn on_delete_fixture_button_click(&self, model: &mut Model, fixture_id: FixtureId) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete this fixture?")) == Ok(true) {
                //TODO: also delete the associated match if any??
                if let Err(_) = model.delete_fixture(tournament_id, stage_id, fixture_id) {
                    error!("Failed to delete fixture");
                }
            }
        }
    }

    fn on_fixture_drag_handle_mousedown(&mut self, model: &mut Model, fixture_id: FixtureId, e: MouseEvent) {
        debug!("Dragging {fixture_id}");
        if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
            let fixture_div_rect = fixture_div.get_bounding_client_rect();
            self.current_drag = Some(DragInfo { fixture_id, start_offset: (e.client_x() as f64 - fixture_div_rect.left(), e.client_y() as f64 - fixture_div_rect.top()) });
        }
    }

    fn on_background_mousemove(&self, model: &mut Model, e: MouseEvent) {
        debug!("move to {} {}", e.offset_x(), e.offset_y());
        if let Some(drag_info) = &self.current_drag {
            if let Some(fixture_div) = self.fixture_divs.get(&drag_info.fixture_id) {
                // Get the cursor position relative to the bracket view
                // This won't be the same as e.offsetX/Y if we are currently dragging over a child element of the bracket view,
                // as we receive the mousemove event via bubbling, so e.target will be the child element, and e.offsetX/Y will be relative to that instead.
              //  let e_target = e.target().and_then(|t| t.dyn_into::<HtmlElement>().ok());
              //  let target_rect =  e_target.as_ref().map_or(DomRect::new().unwrap(), |t| t.get_bounding_client_rect());
                let canvas_rect = self.canvas.get_bounding_client_rect();
                // let x = e.offset_x() as f64 + target_rect.left() - canvas_rect.left();
                // let y = e.offset_y() as f64 + target_rect.top() - canvas_rect.top();
                let x = e.client_x() as f64 - canvas_rect.left() - drag_info.start_offset.0;
                let y = e.client_y() as f64 - canvas_rect.top() - drag_info.start_offset.1;
                // debug!("target rect = {},{}. root rect = {},{}. updated to {x} {y}", target_rect.left(), target_rect.top(), canvas_rect.left(), canvas_rect.top());

                fixture_div.style().set_property("left", &x.to_string()).expect("Failed to set property");
                fixture_div.style().set_property("top", &y.to_string()).expect("Failed to set property");
            }
        }
    }

    fn on_background_mouseup(&mut self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(drag_info) = &self.current_drag {
                let fixture_id = drag_info.fixture_id;
                if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
                    let canvas_rect = self.canvas.get_bounding_client_rect();
                    let x = e.client_x() as f64 - canvas_rect.left() - drag_info.start_offset.0;
                    let y = e.client_y() as f64 - canvas_rect.top() - drag_info.start_offset.1;

                    self.current_drag = None;
                    if let Err(_) = model.set_fixture_layout(tournament_id, stage_id, fixture_id, (x as i32, y as i32)) {
                        error!("Failed to update fixture");
                    }
                }
            }
        }
    }
}
