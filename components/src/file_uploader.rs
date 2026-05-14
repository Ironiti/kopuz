use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[component]
pub fn FileUploader(
    on_files_selected: EventHandler<Vec<PathBuf>>,
    accept_formats: Option<String>,
) -> Element {
    let mut is_uploading = use_signal(|| false);
    let mut upload_progress = use_signal(|| String::new());

    let handle_file_select = move |_| {
        spawn(async move {
            is_uploading.set(true);
            upload_progress.set(i18n::t("selecting_files").to_string());

            let mut dialog = AsyncFileDialog::new()
                .set_title(i18n::t("select_audio_video_files"));

            // Set accepted file formats
            if let Some(formats) = accept_formats {
                dialog = dialog.add_filter("Media Files", &formats.split(',').collect::<Vec<_>>());
            } else {
                // Default: accept all audio and video formats
                dialog = dialog.add_filter(
                    "Audio/Video Files",
                    &[
                        "mp3", "flac", "m4a", "aac", "wav", "ogg", "opus", "wma", "aiff", "ape",
                        "alac", "webm", "mka", "oga", "spx", "tta", "wv", "dts", "ac3", "amr",
                        "mp4", "mkv", "avi", "mov", "wmv", "flv", "m4v", "mpg", "mpeg", "3gp",
                        "ogv", "ts", "vob", "divx", "f4v", "asf", "rm", "rmvb",
                    ],
                );
            }

            let files = dialog.pick_files().await;

            if let Some(files) = files {
                let paths: Vec<PathBuf> = files.iter().map(|f| f.path().to_path_buf()).collect();
                upload_progress.set(format!("{} {}", i18n::t("selected"), paths.len()));
                on_files_selected.call(paths);
            }

            is_uploading.set(false);
            upload_progress.set(String::new());
        });
    };

    rsx! {
        div { class: "file-uploader",
            button {
                class: "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                disabled: is_uploading(),
                onclick: handle_file_select,
                if is_uploading() {
                    span { class: "flex items-center gap-2",
                        svg {
                            class: "animate-spin h-5 w-5",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            circle {
                                class: "opacity-25",
                                cx: "12",
                                cy: "12",
                                r: "10",
                                stroke: "currentColor",
                                stroke_width: "4"
                            }
                            path {
                                class: "opacity-75",
                                fill: "currentColor",
                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            }
                        }
                        "{upload_progress()}"
                    }
                } else {
                    "{i18n::t(\"add_files\")}"
                }
            }
        }
    }
}
