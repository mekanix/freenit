use topcoat::{
    Result,
    context::Cx,
    runtime::Signal,
    view::{View, component, view},
};

#[derive(Clone, Copy, Default)]
pub enum ToastVariant {
    #[default]
    Error,
    Warning,
}

impl ToastVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Error => "toast toast-error",
            Self::Warning => "toast toast-warning",
        }
    }
}

/// A notification rendered by the root layout when attached to a rewrite.
pub struct Notification {
    pub message: String,
    pub variant: ToastVariant,
}

impl Notification {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            variant: ToastVariant::Error,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            variant: ToastVariant::Warning,
        }
    }
}

#[component]
pub async fn toast(
    cx: &Cx,
    open: &Signal<bool>,
    message: &str,
    #[default] variant: ToastVariant,
) -> Result<impl View> {
    Ok(view! {
        <div
            class=(variant.class())
            role="alert"
            aria-live="assertive"
            :hidden=$(!open.get())
        >
            <span class="toast-message">(message)</span>
            <button
                class="toast-close"
                type="button"
                aria-label="Dismiss notification"
                @click=$(|_e| open.set(false))
            >
                "×"
            </button>
        </div>
    })
}
