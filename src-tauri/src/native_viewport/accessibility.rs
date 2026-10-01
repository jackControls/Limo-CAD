//! Accessibility tree for the native shell: title bar, ribbon, and viewport.
//!
//! Names come from strings the shell already paints, including the shared
//! dictionaries. Keyboard focus follows reading order: each horizontal band
//! left to right, then the next band down.
//!
//! UI Automation is not connected. The `windows` crate in this desktop build
//! does not enable `Win32_UI_Accessibility`, and this module does not call
//! that API. [`AccessibilityHost`] is the seam; [`DisconnectedUiAutomation`]
//! only retains the tree in process.

use crate::app_preferences::{locale as dictionary, Locale};
use serde::Deserialize;

/// False because UI Automation is not linked or called from this build.
pub(crate) const UI_AUTOMATION_CONNECTED: bool = false;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Window,
    TitleBar,
    Button,
    TabList,
    Tab,
    Status,
    Toolbar,
    Group,
    Label,
    Pane,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
    pub id: String,
    pub role: Role,
    pub name: String,
    pub bounds: Bounds,
    pub focusable: bool,
    pub disabled: bool,
    pub children: Vec<Node>,
}

impl Node {
    pub(crate) fn walk(&self) -> Vec<&Node> {
        let mut out = Vec::new();
        self.walk_into(&mut out);
        out
    }

    fn walk_into<'a>(&'a self, out: &mut Vec<&'a Node>) {
        out.push(self);
        for child in &self.children {
            child.walk_into(out);
        }
    }

    pub(crate) fn find(&self, id: &str) -> Option<&Node> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tree {
    root: Node,
    focus: Option<String>,
}

impl Tree {
    pub(crate) fn root(&self) -> &Node {
        &self.root
    }

    pub(crate) fn node(&self, id: &str) -> Option<&Node> {
        self.root.find(id)
    }

    pub(crate) fn focused_id(&self) -> Option<&str> {
        self.focus.as_deref()
    }

    /// Focusable, enabled nodes in the order a pointer user reads the shell.
    pub(crate) fn focus_order(&self) -> Vec<&Node> {
        self.focus_ids()
            .iter()
            .filter_map(|id| self.root.find(id))
            .collect()
    }

    pub(crate) fn focus_next(&mut self) -> Option<&str> {
        self.step(1)
    }

    pub(crate) fn focus_previous(&mut self) -> Option<&str> {
        self.step(-1)
    }

    fn focus_ids(&self) -> Vec<String> {
        let mut items = Vec::new();
        collect_focusable(&self.root, &mut items);
        if items.is_empty() {
            return Vec::new();
        }
        items.sort_by(|a, b| {
            a.y.total_cmp(&b.y)
                .then(a.x.total_cmp(&b.x))
                .then(a.id.cmp(&b.id))
        });
        let mut rows: Vec<Vec<FocusItem>> = Vec::new();
        for item in items {
            let joins = rows.last().is_some_and(|row| {
                let bottom = row
                    .iter()
                    .map(|entry| entry.bottom)
                    .fold(f32::MIN, f32::max);
                item.y < bottom
            });
            if joins {
                rows.last_mut().unwrap().push(item);
            } else {
                rows.push(vec![item]);
            }
        }
        let mut ids = Vec::new();
        for mut row in rows {
            row.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.id.cmp(&b.id)));
            ids.extend(row.into_iter().map(|item| item.id));
        }
        ids
    }

    fn step(&mut self, delta: isize) -> Option<&str> {
        let ids = self.focus_ids();
        if ids.is_empty() {
            self.focus = None;
            return None;
        }
        let next = match (
            self.focus
                .as_ref()
                .and_then(|id| ids.iter().position(|candidate| candidate == id)),
            delta >= 0,
        ) {
            (Some(index), true) => (index + 1) % ids.len(),
            (Some(index), false) => (index + ids.len() - 1) % ids.len(),
            (None, true) => 0,
            (None, false) => ids.len() - 1,
        };
        self.focus = Some(ids[next].clone());
        self.focus.as_deref()
    }
}

struct FocusItem {
    id: String,
    x: f32,
    y: f32,
    bottom: f32,
}

fn collect_focusable(node: &Node, out: &mut Vec<FocusItem>) {
    if node.focusable && !node.disabled {
        out.push(FocusItem {
            id: node.id.clone(),
            x: node.bounds.x,
            y: node.bounds.y,
            bottom: node.bounds.bottom(),
        });
    }
    for child in &node.children {
        collect_focusable(child, out);
    }
}

/// Platform accessibility bridge. A host may retain a tree without sending it
/// to assistive technology.
pub(crate) trait AccessibilityHost {
    fn connected(&self) -> bool;
    fn apply(&mut self, tree: &Tree);
}

/// In-process stand-in. `connected` stays false because UI Automation is not linked.
#[derive(Clone, Debug, Default)]
pub(crate) struct DisconnectedUiAutomation {
    retained: Option<Tree>,
}

impl DisconnectedUiAutomation {
    pub(crate) fn retained(&self) -> Option<&Tree> {
        self.retained.as_ref()
    }
}

impl AccessibilityHost for DisconnectedUiAutomation {
    fn connected(&self) -> bool {
        false
    }

    fn apply(&mut self, tree: &Tree) {
        self.retained = Some(tree.clone());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum McpStatus {
    Attached,
    Waiting,
    Off,
}

#[derive(Clone, Debug)]
pub(crate) struct ShellDocument {
    pub name: String,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ShellSnapshot {
    pub locale: Locale,
    pub width: f32,
    pub height: f32,
    pub documents: Vec<ShellDocument>,
    pub mcp: McpStatus,
}

pub(crate) fn fixture_shell() -> ShellSnapshot {
    fixture_shell_in(Locale::En)
}

pub(crate) fn fixture_shell_in(locale: Locale) -> ShellSnapshot {
    ShellSnapshot {
        locale,
        width: 1600.,
        height: 900.,
        documents: vec![
            ShellDocument {
                name: text(locale, "app.untitledDocument"),
                active: false,
            },
            ShellDocument {
                name: "Bracket".into(),
                active: true,
            },
        ],
        mcp: McpStatus::Off,
    }
}

pub(crate) fn shell_tree(shell: &ShellSnapshot) -> Tree {
    let viewport_name = shell
        .documents
        .iter()
        .find(|document| document.active)
        .or(shell.documents.first())
        .map(|document| document.name.clone())
        .unwrap_or_else(|| text(shell.locale, "app.untitledDocument"));
    Tree {
        root: Node {
            id: "window".into(),
            role: Role::Window,
            name: text(shell.locale, "app.name"),
            bounds: Bounds::new(0., 0., shell.width, shell.height),
            focusable: false,
            disabled: false,
            children: vec![
                title_bar(shell),
                ribbon(shell),
                viewport(shell, &viewport_name),
            ],
        },
        focus: None,
    }
}

fn text(locale: Locale, key: &str) -> String {
    dictionary::translate(locale, key).to_owned()
}

fn mcp_label(status: &McpStatus) -> &'static str {
    match status {
        McpStatus::Attached => "MCP attached",
        McpStatus::Waiting => "MCP waiting",
        McpStatus::Off => "MCP off",
    }
}

fn control(
    id: impl Into<String>,
    role: Role,
    name: impl Into<String>,
    bounds: Bounds,
    focusable: bool,
    disabled: bool,
) -> Node {
    Node {
        id: id.into(),
        role,
        name: name.into(),
        bounds,
        focusable: focusable && !disabled,
        disabled,
        children: Vec::new(),
    }
}

fn title_bar(shell: &ShellSnapshot) -> Node {
    let locale = shell.locale;
    let mut children = vec![
        control(
            "file",
            Role::Button,
            text(locale, "file.menu"),
            Bounds::new(0., 0., 40., 28.),
            true,
            false,
        ),
        control(
            "new",
            Role::Button,
            text(locale, "topbar.newDesign"),
            Bounds::new(40., 0., 28., 28.),
            true,
            false,
        ),
    ];
    let available = ((shell.width - 286.) / 192.).floor().max(1.) as usize;
    let active = shell
        .documents
        .iter()
        .position(|document| document.active)
        .unwrap_or(0);
    let start = active.saturating_sub(available.saturating_sub(1));
    let visible: Vec<(usize, &ShellDocument)> = shell
        .documents
        .iter()
        .enumerate()
        .skip(start)
        .take(available)
        .collect();
    if !visible.is_empty() {
        let mut tabs = Vec::new();
        for (offset, (index, document)) in visible.iter().enumerate() {
            let x = 68. + offset as f32 * 192.;
            let close_name = if document.active {
                text(locale, "topbar.closeDocument")
            } else {
                format!(
                    "{}: {}",
                    text(locale, "topbar.closeDocument"),
                    document.name
                )
            };
            tabs.push(control(
                format!("tab-{index}"),
                Role::Tab,
                document.name.clone(),
                Bounds::new(x + 10., 2., 152., 26.),
                true,
                false,
            ));
            tabs.push(control(
                format!("close-tab-{index}"),
                Role::Button,
                close_name,
                Bounds::new(x + 166., 5., 20., 20.),
                true,
                false,
            ));
        }
        children.push(Node {
            id: "open-documents".into(),
            role: Role::TabList,
            name: text(locale, "file.openDocuments"),
            bounds: Bounds::new(68., 0., visible.len() as f32 * 192., 28.),
            focusable: false,
            disabled: false,
            children: tabs,
        });
    }
    if shell.documents.len() > available {
        if active > 0 {
            children.push(control(
                "previous-tab",
                Role::Button,
                "Previous document",
                Bounds::new(shell.width - 142., 0., 26., 28.),
                true,
                false,
            ));
        }
        if active + 1 < shell.documents.len() {
            children.push(control(
                "next-tab",
                Role::Button,
                "Next document",
                Bounds::new(shell.width - 114., 0., 26., 28.),
                true,
                false,
            ));
        }
    }
    children.push(control(
        "mcp",
        Role::Status,
        mcp_label(&shell.mcp),
        Bounds::new(shell.width - 218., 0., 72., 28.),
        true,
        false,
    ));
    children.push(control(
        "scripts",
        Role::Button,
        text(locale, "topbar.scripts"),
        Bounds::new(shell.width - 86., 0., 86., 28.),
        true,
        false,
    ));
    Node {
        id: "title-bar".into(),
        role: Role::TitleBar,
        name: text(locale, "app.name"),
        bounds: Bounds::new(0., 0., shell.width, 28.),
        focusable: false,
        disabled: false,
        children,
    }
}

fn ribbon(shell: &ShellSnapshot) -> Node {
    let locale = shell.locale;
    let workspace_width = if shell.width > 1400. { 108. } else { 56. };
    let catalog: Catalog = serde_json::from_str(include_str!("../../../interface/catalog.json"))
        .expect("interface catalog");
    let panels = catalog
        .workspaces
        .into_iter()
        .find(|workspace| workspace.id == "solid")
        .map(|workspace| workspace.panels)
        .unwrap_or_default();
    let counts = visible_counts(
        &panels
            .iter()
            .map(|panel| panel.buttons.len())
            .collect::<Vec<_>>(),
        shell.width - workspace_width - 6.,
    );
    let mut x = workspace_width;
    let mut panels_nodes = Vec::new();
    for (panel, count) in panels.into_iter().zip(counts) {
        let group_width = 9. + (count as f32 * 50. - 2.).max(48.);
        let has_menu = panel.id != "profile" && panel.id != "selection";
        let group_name = text(locale, &panel.label_key);
        let mut children = Vec::new();
        for (index, button) in panel.buttons.iter().take(count).enumerate() {
            children.push(control(
                format!("tool-{}", button.id),
                Role::Button,
                text(locale, &button.label_key),
                Bounds::new(x + 4. + index as f32 * 50., 34., 48., 52.),
                true,
                false,
            ));
        }
        children.push(control(
            format!("group-{}", panel.id),
            Role::Button,
            group_name.clone(),
            Bounds::new(x + 4., 90., group_width - 9., 20.),
            has_menu,
            !has_menu,
        ));
        panels_nodes.push(Node {
            id: format!("panel-{}", panel.id),
            role: Role::Group,
            name: group_name,
            bounds: Bounds::new(x, 28., group_width, 92.),
            focusable: false,
            disabled: false,
            children,
        });
        x += group_width;
    }
    let mut children = vec![
        control(
            "workspace",
            Role::Button,
            text(locale, "workspace.switchWorkspace"),
            Bounds::new(4., 34., workspace_width - 8., 52.),
            true,
            false,
        ),
        control(
            "workspace-title",
            Role::Label,
            text(locale, "ribbon.panels.workspace"),
            Bounds::new(0., 96., workspace_width, 16.),
            false,
            false,
        ),
    ];
    children.extend(panels_nodes);
    Node {
        id: "ribbon".into(),
        role: Role::Toolbar,
        name: text(locale, "ribbon.tabs.solidModeling"),
        bounds: Bounds::new(0., 28., shell.width, 92.),
        focusable: false,
        disabled: false,
        children,
    }
}

fn viewport(shell: &ShellSnapshot, name: &str) -> Node {
    let side = 240_f32.min(shell.width * 0.45);
    let top = 120_f32.min(shell.height * 0.3);
    let bottom = 48_f32.min(shell.height * 0.1);
    control(
        "viewport",
        Role::Pane,
        name,
        Bounds::new(
            side,
            top,
            (shell.width - side).max(0.),
            (shell.height - bottom - top).max(0.),
        ),
        true,
        false,
    )
}

/// Same shedding rule as the solid ribbon: drop secondary commands until the
/// groups fit, keeping each group's primary command.
fn visible_counts(button_counts: &[usize], available: f32) -> Vec<usize> {
    let mut counts = button_counts.to_vec();
    let total = |counts: &[usize]| {
        counts
            .iter()
            .map(|count| 9. + (*count as f32 * 50. - 2.).max(48.))
            .sum::<f32>()
    };
    let max = counts.iter().copied().max().unwrap_or(0);
    for slot in (1..max).rev() {
        for index in (0..counts.len()).rev() {
            if total(&counts) <= available {
                return counts;
            }
            if counts[index] > slot {
                counts[index] -= 1;
            }
        }
    }
    counts
}

#[derive(Deserialize)]
struct Catalog {
    workspaces: Vec<CatalogWorkspace>,
}

#[derive(Deserialize)]
struct CatalogWorkspace {
    id: String,
    panels: Vec<CatalogPanel>,
}

#[derive(Deserialize)]
struct CatalogPanel {
    id: String,
    #[serde(rename = "labelKey")]
    label_key: String,
    #[serde(default)]
    buttons: Vec<CatalogButton>,
}

#[derive(Deserialize)]
struct CatalogButton {
    id: String,
    #[serde(rename = "labelKey")]
    label_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(tree: &Tree, id: &str) -> (Role, String) {
        let node = tree.node(id).unwrap_or_else(|| panic!("missing {id}"));
        (node.role, node.name.clone())
    }

    fn focus_ids(tree: &Tree) -> Vec<String> {
        tree.focus_order()
            .into_iter()
            .map(|node| node.id.clone())
            .collect()
    }

    #[test]
    fn fixture_shell_exposes_roles_names_and_visual_focus_order() {
        let shell = fixture_shell();
        let mut tree = shell_tree(&shell);
        let locale = shell.locale;
        let label = |key| text(locale, key);

        for node in tree.root().walk() {
            assert!(!node.name.is_empty(), "{}", node.id);
        }
        assert_eq!(
            tree.root()
                .children
                .iter()
                .map(|child| child.id.as_str())
                .collect::<Vec<_>>(),
            vec!["title-bar", "ribbon", "viewport"]
        );

        assert_eq!(names(&tree, "window"), (Role::Window, label("app.name")));
        assert_eq!(
            names(&tree, "title-bar"),
            (Role::TitleBar, label("app.name"))
        );
        assert_eq!(names(&tree, "file"), (Role::Button, label("file.menu")));
        assert_eq!(
            names(&tree, "new"),
            (Role::Button, label("topbar.newDesign"))
        );
        assert_eq!(
            names(&tree, "open-documents"),
            (Role::TabList, label("file.openDocuments"))
        );
        assert_eq!(
            names(&tree, "tab-0"),
            (Role::Tab, label("app.untitledDocument"))
        );
        assert_eq!(
            names(&tree, "close-tab-0"),
            (
                Role::Button,
                format!(
                    "{}: {}",
                    label("topbar.closeDocument"),
                    label("app.untitledDocument")
                )
            )
        );
        assert_eq!(names(&tree, "tab-1"), (Role::Tab, "Bracket".into()));
        assert_eq!(
            names(&tree, "close-tab-1"),
            (Role::Button, label("topbar.closeDocument"))
        );
        assert_eq!(names(&tree, "mcp"), (Role::Status, "MCP off".into()));
        assert_eq!(
            names(&tree, "scripts"),
            (Role::Button, label("topbar.scripts"))
        );
        assert_eq!(
            names(&tree, "ribbon"),
            (Role::Toolbar, label("ribbon.tabs.solidModeling"))
        );
        assert_eq!(
            names(&tree, "workspace"),
            (Role::Button, label("workspace.switchWorkspace"))
        );
        assert_eq!(
            names(&tree, "workspace-title"),
            (Role::Label, label("ribbon.panels.workspace"))
        );
        assert!(!tree.node("workspace-title").unwrap().focusable);
        assert_eq!(
            names(&tree, "tool-createSketch"),
            (Role::Button, label("ribbon.solid.createSketch"))
        );
        assert_eq!(
            names(&tree, "tool-extrude"),
            (Role::Button, label("ribbon.solid.extrude"))
        );
        assert_eq!(
            names(&tree, "tool-select"),
            (Role::Button, label("ribbon.solid.select"))
        );
        assert_eq!(
            names(&tree, "group-profile"),
            (Role::Button, label("ribbon.panels.profile"))
        );
        assert!(tree.node("group-profile").unwrap().disabled);
        assert_eq!(
            names(&tree, "group-build"),
            (Role::Button, label("ribbon.panels.build"))
        );
        assert!(!tree.node("group-build").unwrap().disabled);
        assert!(tree.node("group-selection").unwrap().disabled);
        assert_eq!(names(&tree, "viewport"), (Role::Pane, "Bracket".into()));
        assert_eq!(tree.node("viewport").unwrap().bounds.y, 120.);
        assert!(
            tree.node("tool-extrude").unwrap().bounds.x
                > tree.node("tool-createSketch").unwrap().bounds.x
        );
        assert!(
            tree.node("group-build").unwrap().bounds.y
                > tree.node("tool-extrude").unwrap().bounds.y
        );

        let order = focus_ids(&tree);
        assert!(!order
            .iter()
            .any(|id| id == "group-profile" || id == "group-selection"));
        assert_eq!(
            order,
            vec![
                "file",
                "new",
                "tab-0",
                "close-tab-0",
                "tab-1",
                "close-tab-1",
                "mcp",
                "scripts",
                "workspace",
                "tool-createSketch",
                "tool-extrude",
                "tool-revolve",
                "tool-sweep",
                "tool-loft",
                "tool-rib",
                "tool-hole",
                "tool-externalThread",
                "tool-fillet",
                "tool-chamfer",
                "tool-shell",
                "tool-mirror",
                "tool-patternRectangular",
                "tool-patternCircular",
                "tool-moveCopy",
                "tool-combine",
                "tool-splitBody",
                "tool-constructionVisibility",
                "tool-offsetPlane",
                "tool-midplane",
                "tool-measure",
                "tool-sectionAnalysis",
                "tool-assemblyBrowser",
                "tool-createJoint",
                "tool-select",
                "group-build",
                "group-refine",
                "group-repeat",
                "group-body",
                "group-reference",
                "group-check",
                "group-assembly",
                "viewport",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
        );

        assert!(tree.focused_id().is_none());
        for id in &order {
            assert_eq!(tree.focus_next(), Some(id.as_str()));
            assert_eq!(tree.focused_id(), Some(id.as_str()));
        }
        assert_eq!(tree.focus_next(), Some(order[0].as_str()));
        assert_eq!(tree.focus_previous(), Some(order.last().unwrap().as_str()));
    }

    #[test]
    fn localized_shell_keeps_painted_mcp_text_and_the_same_focus_order() {
        let english = focus_ids(&shell_tree(&fixture_shell()));
        let shell = fixture_shell_in(Locale::De);
        let tree = shell_tree(&shell);
        assert_eq!(
            tree.node("file").unwrap().name,
            dictionary::translate(Locale::De, "file.menu")
        );
        assert_eq!(
            tree.node("scripts").unwrap().name,
            dictionary::translate(Locale::De, "topbar.scripts")
        );
        assert_eq!(
            tree.node("tab-0").unwrap().name,
            dictionary::translate(Locale::De, "app.untitledDocument")
        );
        assert_eq!(tree.node("mcp").unwrap().name, "MCP off");
        assert_eq!(focus_ids(&tree), english);
    }

    #[test]
    fn overflow_tabs_place_previous_and_next_where_the_title_bar_paints_them() {
        let mut shell = fixture_shell();
        shell.width = 500.;
        shell.documents.push(ShellDocument {
            name: "Plate".into(),
            active: false,
        });
        shell.documents[1].active = false;
        shell.documents[2].active = true;
        let tree = shell_tree(&shell);
        let title: Vec<_> = focus_ids(&tree)
            .into_iter()
            .take_while(|id| id != "workspace")
            .collect();
        assert_eq!(
            title,
            vec![
                "file",
                "new",
                "tab-2",
                "close-tab-2",
                "mcp",
                "previous-tab",
                "scripts",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
        );
        assert_eq!(tree.node("previous-tab").unwrap().name, "Previous document");
        assert!(tree.node("next-tab").is_none());
        assert_eq!(
            tree.node("close-tab-2").unwrap().name,
            text(Locale::En, "topbar.closeDocument")
        );
    }

    #[test]
    fn ui_automation_host_retains_the_tree_without_connecting() {
        let tree = shell_tree(&fixture_shell());
        let mut host = DisconnectedUiAutomation::default();
        assert!(!UI_AUTOMATION_CONNECTED);
        assert!(!host.connected());
        host.apply(&tree);
        assert!(!host.connected());
        let retained = host.retained().unwrap();
        assert_eq!(retained.node("title-bar").unwrap().role, Role::TitleBar);
        assert_eq!(retained.node("ribbon").unwrap().role, Role::Toolbar);
        assert_eq!(retained.node("viewport").unwrap().role, Role::Pane);
        assert_eq!(retained.node("viewport").unwrap().name, "Bracket");
    }
}
