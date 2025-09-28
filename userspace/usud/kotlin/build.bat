@echo off
REM Download and setup Gradle wrapper if not present
if not exist "gradle\wrapper\gradle-wrapper.jar" (
    echo Downloading Gradle wrapper...
    powershell -Command "Invoke-WebRequest -Uri https://raw.githubusercontent.com/gradle/gradle/master/gradle/wrapper/gradle-wrapper.jar -OutFile gradle\wrapper\gradle-wrapper.jar"
)

REM Build the APK
echo Building UserSU APK...
.\gradlew.bat assembleDebug

echo.
echo If build is successful, APK will be in:
echo build\outputs\apk\debug\UserSU-debug.apk