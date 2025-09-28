use std::process::Command;
use std::path::Path;
use std::fs;
use std::io;

fn download_gradle_wrapper() -> io::Result<()> {
    let wrapper_path = Path::new("kotlin/gradle/wrapper/gradle-wrapper.jar");
    let wrapper2_path = Path::new("java/gradle/wrapper/gradle-wrapper.jar");
    if !wrapper_path.exists() {
        println!("Downloading Gradle wrapper...");
        fs::create_dir_all(wrapper_path.parent().unwrap())?;
        
        #[cfg(target_os = "windows")]
        let download_result = Command::new("powershell")
            .args([
                "-Command",
                "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/gradle/gradle/master/gradle/wrapper/gradle-wrapper.jar' -OutFile 'kotlin/gradle/wrapper/gradle-wrapper.jar'"
            ])
            .status()?;

        #[cfg(not(target_os = "windows"))]
        let download_result = Command::new("curl")
            .args([
                "-L",
                "https://raw.githubusercontent.com/gradle/gradle/master/gradle/wrapper/gradle-wrapper.jar",
                "-o",
                "kotlin/gradle/wrapper/gradle-wrapper.jar"
            ])
            .status()?;

        if !download_result.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to download Gradle wrapper"
            ));
        }
    }
    Ok(())
}

fn build_android() -> io::Result<()> {
    // Ensure we're in the right directory context
    let kotlin_dir = Path::new("kotlin");
    if !kotlin_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "kotlin directory not found"
        ));
    }

    // Download Gradle wrapper if needed
    download_gradle_wrapper()?;

    // Build the Android app
    println!("Building UserSU Android app...");
    
    #[cfg(target_os = "windows")]
    let gradle_command = "gradlew.bat";
    #[cfg(not(target_os = "windows"))]
    let gradle_command = "./gradlew";

    let build_result = Command::new(gradle_command)
        .current_dir(kotlin_dir)
        .arg("assembleDebug")
        .status()?;

    if build_result.success() {
        println!("\nBuild successful!");
        println!("APK location: kotlin/build/outputs/apk/debug/UserSU-debug.apk");
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Build failed"
        ))
    }
}

fn main() {
    if let Err(e) = build_android() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}