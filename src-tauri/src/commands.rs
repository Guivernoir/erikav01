use crate::preprocessing::{
    router::{Mode, Proficiency, Personality},
    context::Context,
    cleaner::Cleaner,
};
use tauri::command;
use crate::llama;
use sys_locale::get_locale;

#[tauri::command]
fn get_system_language() -> String {
    get_locale().unwrap_or_else(|| "en-US".to_string())
}

// State management
pub struct AppState {
    pub mode: Option<Mode>,
    pub proficiency: Option<Proficiency>,
    pub personality: Option<Personality>,
    pub context: Option<Context>,
}

#[tauri::command]
pub async fn receive_input(
    input: String,
    llm: tauri::State<'_, llama::LLMEngine>,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    let cleaned = Cleaner::clean(&input)
        .map_err(|e| format!("Input cleaning failed: {}", e))?;
    
    let context = Context::analyze(cleaned, &llm).await
        .map_err(|e| format!("Context analysis failed: {}", e))?;
    
    state.lock().unwrap().context = Some(context);
    Ok("Input analyzed successfully".to_string())
}

#[tauri::command]
pub async fn receive_mode(
    mode: u32,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    let mode = Mode::select_mode(mode).await?;
    state.lock().unwrap().mode = Some(mode);
    Ok("Mode set successfully".to_string())
}

#[tauri::command]
pub async fn receive_proficiency(
    proficiency: u32,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    let prof = Proficiency::select_proficiency(proficiency).await?;
    state.lock().unwrap().proficiency = Some(prof);
    Ok("Proficiency set successfully".to_string())
}

#[tauri::command]
pub async fn receive_personality(
    personality: u32,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    let pers = Personality::select_personality(personality).await?;
    state.lock().unwrap().personality = Some(pers);
    Ok("Personality set successfully".to_string())
}

// FIXED: Now accepts and processes all parameters in one command
#[tauri::command]
pub async fn send_output(
    input: String,
    mode: u32,
    proficiency: u32,
    personality: u32,
    llm: tauri::State<'_, llama::LLMEngine>,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    // Process input and set parameters
    let cleaned = Cleaner::clean(&input)
        .map_err(|e| format!("Input cleaning failed: {}", e))?;
    
    let context = Context::analyze(cleaned, &llm).await
        .map_err(|e| format!("Context analysis failed: {}", e))?;
    
    let mode_enum = Mode::select_mode(mode).await?;
    let proficiency_enum = Proficiency::select_proficiency(proficiency).await?;
    let personality_enum = Personality::select_personality(personality).await?;
    
    // Update state with all parameters
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.context = Some(context);
        state_guard.mode = Some(mode_enum);
        state_guard.proficiency = Some(proficiency_enum);
        state_guard.personality = Some(personality_enum);
    }
    
    // Generate response based on stored context and parameters
    let state_guard = state.lock().unwrap();
    let context = state_guard.context.as_ref()
        .ok_or("No context available")?;
    
    llm.generate(&context.raw_input, None)
        .map_err(|e| e.to_string())
}