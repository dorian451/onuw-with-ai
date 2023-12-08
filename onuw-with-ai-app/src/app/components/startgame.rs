use hashlink::LinkedHashMap;
use leptos::*;
use onuw_game::role::roles::{RoleDef, ROLES};
use std::iter::repeat_with;

#[derive(Clone, Copy)]
struct ActiveRoles(RwSignal<LinkedHashMap<StoredValue<RoleDef>, usize>>);

#[component]
pub fn GameConfig(
    #[prop(default = Callback::new(|_|{();}), into)] on_click: Callback<
        Vec<(StoredValue<RoleDef>, usize)>,
    >,
    #[prop(default = "Start new game", into)] new_game_label: &'static str,
) -> impl IntoView {
    let inactive_roles =
        create_rw_signal(ROLES.keys().cloned().map(store_value).collect::<Vec<_>>()).read_only();
    let active_roles = create_rw_signal(LinkedHashMap::new());
    let total_roles = create_rw_signal(0);

    provide_context(ActiveRoles(active_roles));

    view! {
        <div class="flex flex-col justify-center p-5 gap-2">
            <div class="flex max-w-full">
                <RolesView label="Available Roles:">
                    {move || {
                        inactive_roles()
                            .into_iter()
                            .map(|v| view! { <RenderRole role=v/> })
                            .collect_view()
                    }}

                </RolesView>

                <RolesView label="Selected Roles:">
                    {move || {
                        let mut c = 0;
                        let v = active_roles()
                            .iter()
                            .flat_map(|(role, amt)| {
                                repeat_with(|| view! { <RenderRole active=true role=*role/> })
                                    .take(*amt)
                            })
                            .inspect(|_| c += 1)
                            .collect_view();
                        total_roles.set(c);
                        v
                    }}

                </RolesView>
            </div>
            <div class="flex flex-col justify-center gap-1 max-w-fit mx-auto">
                <p class="text-center">"Players: " {move || (total_roles() as isize) - 3}</p>
                <button
                    class="bg-green-500 dark:bg-green-700"
                    disabled=move || 4.gt(&total_roles())
                    on:click=move |_| { start_game(active_roles, on_click) }
                >

                    {new_game_label}
                </button>
            </div>
        </div>
    }
}
#[component]
fn RolesView(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex-grow basis-0 flex flex-col border-slate-200">
            <h2 class="text-center">{label}</h2>
            <div class="flex flex-wrap gap-1 justify-center">{children()}</div>
        </div>
    }
}

#[component]
fn RenderRole(#[prop(default = false)] active: bool, role: StoredValue<RoleDef>) -> impl IntoView {
    let active_roles = use_context::<ActiveRoles>().unwrap();
    view! {
        <div on:click=move |_| {
            active_roles
                .0
                .update(|v| if !active { add_role(role, v) } else { remove_role(role, v) })
        }>
            <img
                class="w-32"
                src=role.with_value(|v| format!("/assets/img/roles/{}.png", v.name.to_lowercase()))
            />
        </div>
    }
}

fn add_role(role: StoredValue<RoleDef>, v: &mut LinkedHashMap<StoredValue<RoleDef>, usize>) {
    if v.contains_key(&role) {
        let c = v.get_mut(&role).unwrap();

        role.with_value(|role| *c = (*c + 1).clamp(role.min_amt, role.max_amt));
    } else {
        role.with_value(|rolev| {
            v.insert(role, rolev.min_amt);
        });
    }
}

fn remove_role(role: StoredValue<RoleDef>, v: &mut LinkedHashMap<StoredValue<RoleDef>, usize>) {
    if v.contains_key(&role) {
        let c = v.get_mut(&role).unwrap();
        *c -= 1;

        let c = *c;

        role.with_value(|rolev| {
            let b = c < rolev.min_amt;
            if b {
                v.remove(&role);
            }
        })
    } else {
        v.insert(role, 1);
    }
}

fn start_game(
    active_roles: RwSignal<LinkedHashMap<StoredValue<RoleDef>, usize>>,
    on_click: Callback<Vec<(StoredValue<RoleDef>, usize)>>,
) {
    let selected_roles: Vec<_> = active_roles
        .get()
        .iter()
        .map(|(role, amt)| (*role, *amt))
        .collect();

    on_click(selected_roles);
}

