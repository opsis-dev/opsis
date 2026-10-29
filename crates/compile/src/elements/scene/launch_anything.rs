use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LaunchAnythingLogoImage {
    #[setting(
        "Image",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
}

#[settings(export)]
pub struct LaunchAnythingStat {
    #[setting("Label", default = "Label")]
    pub label: String,
    #[setting("Value", default = "0")]
    pub value: String,
}

fn stat(label: &str, value: &str) -> LaunchAnythingStat {
    LaunchAnythingStat {
        label: label.into(),
        value: value.into(),
    }
}

#[settings(export)]
pub struct LaunchAnything {
    #[setting("Opening: first word", default = "Good")]
    pub opening: String,
    #[setting("Opening: second word", default = "ideas")]
    pub subject: String,
    #[setting("Closing: web address", default = "yourproduct.example")]
    pub brand_url: String,
    #[setting("Accent colour", default = "#245744", check = colour)]
    pub accent_color: String,
    #[setting("Opening: after the subject", default = "deserve a stage")]
    pub copy_ready: String,
    #[setting("Action: lead-in", default = "Let’s make")]
    pub copy_next: String,
    #[setting("Action: button text", default = "them real")]
    pub copy_action: String,
    #[setting(
        "Proof: headline",
        default = "Your work.\nA clearer view.",
        widget = "textarea"
    )]
    pub copy_proof: String,
    #[setting("Industry: headline", default = "Built around you")]
    pub copy_industry: String,
    #[setting("Industry: promise", default = "Made to be yours.")]
    pub copy_promise: String,
    #[setting("Closing: call to action", default = "Begin")]
    pub copy_launch: String,
    #[setting(
        "Shop screen: headline",
        default = "Small things. Well made.",
        advanced
    )]
    pub copy_shoppers: String,
    #[setting("Shop screen: shop name", default = "Object Study", advanced)]
    pub copy_shop_name: String,
    #[setting(
        "Shop screen: shop headline",
        default = "Everyday, considered.",
        advanced
    )]
    pub copy_shop_headline: String,
    #[setting(
        "Shop screen: shop detail",
        default = "Useful objects. Quiet details.",
        advanced
    )]
    pub copy_shop_detail: String,
    #[setting("Shop screen: shop button", default = "Explore the objects", advanced)]
    pub copy_shop_action: String,
    #[setting(
        "Builder screen: headline",
        default = "Turn a brief into a workspace",
        advanced
    )]
    pub copy_builder: String,
    #[setting(
        "Builder screen: prompt",
        default = "Create a workspace for our next product release",
        advanced
    )]
    pub copy_builder_prompt: String,
    #[setting(
        "Review screen: headline",
        default = "Review once.\nMove forward.",
        widget = "textarea",
        advanced
    )]
    pub copy_trading: String,
    #[setting("Ideas screen: headline", default = "A place for every idea", advanced)]
    pub copy_loyalty: String,
    #[setting("Ideas screen: payoff", default = "Room for the next one", advanced)]
    pub copy_loyalty_payoff: String,
    #[setting(
        "Tools screen: headline",
        default = "Bring your work together",
        advanced
    )]
    pub copy_integration: String,
    #[setting(
        "Tools screen: closing line",
        default = "Connect your tools.\nKeep work moving.",
        widget = "textarea",
        advanced
    )]
    pub copy_connections: String,
    #[setting(
        "Tools screen: tools",
        default = vec!["Files".into(), "Messages".into(), "Calendar".into()],
        advanced
    )]
    pub copy_integrations: Vec<String>,
    #[setting(
        "Screen: dashboard",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_dashboard: String,
    #[setting(
        "Screen: shoppers",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_shoppers: String,
    #[setting(
        "Screen: exchange",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_exchange: String,
    #[setting(
        "Screen: builder",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_builder: String,
    #[setting(
        "Screen: trading",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_trading: String,
    #[setting(
        "Screen: loyalty",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_loyalty: String,
    #[setting(
        "Screen: integrations",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_integrations: String,
    #[setting(
        "Photo: desk",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_desk: String,
    #[setting(
        "Photo: horizon",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_horizon: String,
    #[setting(
        "Photo: rocket",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_rocket: String,
    #[setting(
        "Photo: earth",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_earth: String,
    #[setting("Logos", help = "Marks for the floating grid in the proof scene.")]
    pub logo_images: Vec<LaunchAnythingLogoImage>,
    #[setting(
        "Logo",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in mark"
    )]
    pub logo_src: String,
    #[setting(
        "Music",
        zero_label = "None",
        help = "Plays through the Mixer like any other music.",
        widget = "sound"
    )]
    pub music: String,
    #[setting(
        "Music volume",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub music_volume: f64,
    #[setting(
        "Font",
        default = "Manrope",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting("Night colour", default = "#122d24", check = colour, advanced)]
    pub night_color: String,
    #[setting("Paper colour", default = "#faf6ec", check = colour, advanced)]
    pub paper_color: String,
    #[setting("Light text colour", default = "#fff9f3", check = colour, advanced)]
    pub light_text_color: String,
    #[setting("Ink colour", default = "#17392b", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Sage colour", default = "#497761", check = colour, advanced)]
    pub sage_color: String,
    #[setting(
        "Opening shimmer",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        help = "How strongly the light runs over the opening words.",
        advanced
    )]
    pub opening_shimmer: f64,
    #[setting(
        "Opening word size",
        default = 430,
        range(min = 100, max = 600),
        step = 10,
        unit = "px",
        help = "The second word is drawn a little smaller.",
        advanced
    )]
    pub opening_size: u32,
    #[setting(
        "Headline weight",
        default = 500,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub headline_weight: u32,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        help = "Scales every headline together.",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Photo brightness",
        default = 0.66,
        range(min = 0.1, max = 1.5),
        step = 0.01,
        help = "Behind the opening line; the space scene follows.",
        advanced
    )]
    pub photo_dimming: f64,
    #[setting(
        "Closing logo size",
        default = 64,
        range(min = 16, max = 160),
        step = 1,
        unit = "px",
        advanced
    )]
    pub logo_size: u32,
    #[setting("Dashboard screen: app name", default = "Workroom", advanced)]
    pub copy_app_name: String,
    #[setting(
        "Dashboard screen: menu",
        default = vec![
            "Workspace".into(),
            "Overview".into(),
            "Milestones".into(),
            "Projects".into(),
            "Teammates".into(),
            "Reports".into(),
            "Settings".into(),
        ],
        advanced
    )]
    pub copy_dashboard_menu: Vec<String>,
    #[setting(
        "Dashboard screen: headline",
        default = "Your week, in focus",
        advanced
    )]
    pub copy_dashboard: String,
    #[setting("Dashboard screen: period", default = "This month", advanced)]
    pub copy_dashboard_period: String,
    #[setting(
        "Dashboard screen: card note",
        default = "Demo workspace · this month",
        advanced
    )]
    pub copy_dashboard_note: String,
    #[setting(
        "Dashboard screen: numbers",
        default = vec![
            stat("Tasks closed", "128"),
            stat("Active projects", "12"),
            stat("On track", "92%"),
        ],
        length(min = 1, max = 4),
        advanced
    )]
    pub dashboard_stats: Vec<LaunchAnythingStat>,
    #[setting(
        "Dashboard screen: chart title",
        default = "Project momentum",
        advanced
    )]
    pub copy_momentum: String,
    #[setting(
        "Dashboard screen: activity title",
        default = "Recent activity",
        advanced
    )]
    pub copy_activity_title: String,
    #[setting(
        "Dashboard screen: activity",
        default = vec![
            "Brief approved".into(),
            "Review completed".into(),
            "Project completed".into(),
            "Files organized".into(),
            "Milestone reached".into(),
        ],
        advanced
    )]
    pub dashboard_activity: Vec<String>,
    #[setting(
        "Shop screen: note",
        default = "A collection for slower mornings.",
        advanced
    )]
    pub copy_shop_note: String,
    #[setting("Review screen: list title", default = "Review activity", advanced)]
    pub copy_review_title: String,
    #[setting("Review screen: project", default = "Autumn release", advanced)]
    pub copy_review_project: String,
    #[setting(
        "Review screen: buttons",
        default = vec![
            "Approve".into(),
            "Revise".into(),
        ],
        length(min = 2, max = 2),
        advanced
    )]
    pub copy_review_buttons: Vec<String>,
    #[setting(
        "Review screen: rows",
        default = vec![
            stat("Files reviewed", "12 files"),
            stat("Comments resolved", "28 / 28"),
            stat("Contributors", "4 people"),
            stat("Next milestone", "Ready to share"),
        ],
        advanced
    )]
    pub review_rows: Vec<LaunchAnythingStat>,
    #[setting("Ideas screen: card title", default = "Project notes", advanced)]
    pub copy_ideas_card: String,
    #[setting(
        "Ideas screen: card note",
        default = "Collect ideas. Connect the details.",
        advanced
    )]
    pub copy_ideas_note: String,
}

pub fn definition() -> ElementMeta {
    meta::<LaunchAnything>(
            element(
                ElementKind::Component,
                "launch-anything",
                "Launch anything",
                "An independent 26.7-second editorial product film with forest-green accents, original photography, seven editable demos, and geometric marks.",
                ElementCategory::Scene,
                600,
            )
                .fonts(&["Manrope"])
                .look(NodeLook::of("lucide:film")),
        )
}
