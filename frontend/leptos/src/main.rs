//! Leptos UI for ELXR kombucha telemetry

use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div class="container mx-auto p-4">
            <h1 class="text-2xl font-bold mb-4">"ELXR Kombucha Telemetry"</h1>
            <div class="bg-white rounded-lg shadow-md p-6">
                <h2 class="text-xl font-semibold mb-3">"Fermentation Status"</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="bg-blue-50 p-4 rounded-md">
                        <h3 class="font-medium">"Temperature"</h3>
                        <p class="text-2xl">"24.5Â°C"</p>
                    </div>
                    <div class="bg-green-50 p-4 rounded-md">
                        <h3 class="font-medium">"pH Level"</h3>
                        <p class="text-2xl">"3.2"</p>
                    </div>
                    <div class="bg-purple-50 p-4 rounded-md">
                        <h3 class="font-medium">"Fermentation Time"</h3>
                        <p class="text-2xl">"7 days"</p>
                    </div>
                    <div class="bg-yellow-50 p-4 rounded-md">
                        <h3 class="font-medium">"SCOBY Health"</h3>
                        <p class="text-2xl">"Excellent"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
