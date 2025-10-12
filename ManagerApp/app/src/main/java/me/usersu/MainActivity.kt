package me.usersu

import android.content.Context
import android.net.Uri
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Code
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import me.usersu.ui.theme.UserSUTheme
import java.io.File

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            UserSUTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    ManagerAppContent()
                }
            }
        }
    }
}

@Composable
fun ManagerAppContent() {
    val context = LocalContext.current
    var selectedTab by rememberSaveable { mutableStateOf(0) }

    var installStatus by remember { mutableStateOf("Ready to install UserSU") }
    var commandInput by remember { mutableStateOf("") }
    var commandOutput by remember { mutableStateOf("Command Output:") }
    var usudVersion by remember { mutableStateOf("Unknown") }
    var libfakerootStatus by remember { mutableStateOf("Unknown") }

    val pickFileLauncher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) {
        uri: Uri? ->
        uri?.let {
            installStatus = "Installing..."
            val targetDir = File(context.filesDir, "usersu_binaries")
            try {
                context.contentResolver.openInputStream(it)?.use { inputStream ->
                    val success = UserSUInstaller.extractAndInstall(context, inputStream, targetDir)
                    if (success) {
                        installStatus = "UserSU installed successfully to ${targetDir.absolutePath}"
                        Toast.makeText(context, "Installation successful!", Toast.LENGTH_LONG).show()
                    } else {
                        installStatus = "Installation failed."
                        Toast.makeText(context, "Installation failed.", Toast.LENGTH_LONG).show()
                    }
                }
            } catch (e: Exception) {
                installStatus = "Error: ${e.message}"
                Toast.makeText(context, "Error: ${e.message}", Toast.LENGTH_LONG).show()
                e.printStackTrace()
            }
        }
    }

    Scaffold(
        bottomBar = {
            NavigationBar {
                val items = listOf("Install", "Command", "Info", "Uninstall")
                val icons = listOf(Icons.Filled.Build, Icons.Filled.Code, Icons.Filled.Info, Icons.Filled.Delete)
                items.forEachIndexed { index, item ->
                    NavigationBarItem(
                        icon = { Icon(icons[index], contentDescription = item) },
                        label = { Text(item) },
                        selected = selectedTab == index,
                        onClick = { selectedTab = index }
                    )
                }
            }
        }
    ) { paddingValues ->
        Column(modifier = Modifier.padding(paddingValues)) {
            when (selectedTab) {
                0 -> InstallUpdateScreen(installStatus, pickFileLauncher)
                1 -> CommandRunnerScreen(commandInput, commandOutput, { commandInput = it }, { newOutput -> commandOutput = newOutput }, context)
                2 -> VersionSystemInfoScreen(usudVersion, libfakerootStatus, context, { version -> usudVersion = version }, { status -> libfakerootStatus = status })
                3 -> UninstallScreen(installStatus, { newStatus -> installStatus = newStatus }, { version -> usudVersion = version }, { status -> libfakerootStatus = status }, context)
            }
        }
    }
}

@Composable
fun InstallUpdateScreen(installStatus: String, pickFileLauncher: ActivityResultLauncher<Array<String>>) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(text = installStatus)
        Spacer(modifier = Modifier.height(16.dp))
        Button(
            onClick = {
                pickFileLauncher.launch(arrayOf("application/x-tar", "application/gzip", "application/x-xz"))
            }
        ) {
            Text("Select UserSU Tarball and Install/Update")
        }
    }
}

@Composable
fun CommandRunnerScreen(commandInput: String, commandOutput: String, onCommandInputChange: (String) -> Unit, onCommandOutputChange: (String) -> Unit, context: Context) {
    val coroutineScope = rememberCoroutineScope()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(text = "Command Runner")
        OutlinedTextField(
            value = commandInput,
            onValueChange = onCommandInputChange,
            label = { Text("Enter command") },
            modifier = Modifier.fillMaxSize()
        )
        Spacer(modifier = Modifier.height(8.dp))
        Button(
            onClick = {
                onCommandOutputChange("Executing: $commandInput\n(Output will appear here)")
                coroutineScope.launch {
                    val output = withContext(Dispatchers.IO) {
                        UserSUCommandRunner.executeUserSUCommand(context, commandInput)
                    }
                    onCommandOutputChange("Command Output:\n$output")
                }
            }
        ) {
            Text("Execute Command")
        }
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = commandOutput)
    }
}

@Composable
fun VersionSystemInfoScreen(usudVersion: String, libfakerootStatus: String, context: Context, onUsudVersionChange: (String) -> Unit, onLibfakerootStatusChange: (String) -> Unit) {
    val coroutineScope = rememberCoroutineScope()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(text = "UserSU Version Checker")
        Text(text = "usud Version: $usudVersion")
        Text(text = "libfakeroot.so Status: $libfakerootStatus")
        Spacer(modifier = Modifier.height(8.dp))
        Button(
            onClick = {
                coroutineScope.launch {
                    val usudPath = File(context.filesDir, "usersu_binaries/usud")
                    if (usudPath.exists() && usudPath.canExecute()) {
                        val output = withContext(Dispatchers.IO) {
                            UserSUCommandRunner.executeUserSUCommand(context, "--version")
                        }
                        onUsudVersionChange(output.trim())
                    } else {
                        onUsudVersionChange("Not found or not executable")
                    }

                    val libfakerootPath = File(context.filesDir, "usersu_binaries/libfakeroot.so")
                    onLibfakerootStatusChange(if (libfakerootPath.exists()) "Found" else "Not Found")
                }
            }
        ) {
            Text("Check UserSU Versions")
        }

        Spacer(modifier = Modifier.height(32.dp))

        Text(text = "System Information")
        Text(text = "Android Version: ${android.os.Build.VERSION.RELEASE}")
        Text(text = "Device Model: ${android.os.Build.MODEL}")
        Text(text = "UserSU Sandbox Path: ${context.filesDir.absolutePath}/usersu_binaries/fs")
    }
}

@Composable
fun UninstallScreen(installStatus: String, onInstallStatusChange: (String) -> Unit, onUsudVersionChange: (String) -> Unit, onLibfakerootStatusChange: (String) -> Unit, context: Context) {
    val coroutineScope = rememberCoroutineScope()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(text = "Uninstall UserSU")
        Button(
            onClick = {
                coroutineScope.launch {
                    val targetDir = File(context.filesDir, "usersu_binaries")
                    if (targetDir.exists()) {
                        val deleted = withContext(Dispatchers.IO) {
                            targetDir.deleteRecursively()
                        }
                        if (deleted) {
                            onInstallStatusChange("UserSU uninstalled successfully.")
                            Toast.makeText(context, "Uninstallation successful!", Toast.LENGTH_LONG).show()
                        } else {
                            onInstallStatusChange("Uninstallation failed.")
                            Toast.makeText(context, "Uninstallation failed.", Toast.LENGTH_LONG).show()
                        }
                    } else {
                        onInstallStatusChange("UserSU not found for uninstallation.")
                        Toast.makeText(context, "UserSU not found.", Toast.LENGTH_LONG).show()
                    }
                    onUsudVersionChange("Unknown")
                    onLibfakerootStatusChange("Unknown")
                }
            }
        ) {
            Text("Uninstall UserSU")
        }
    }
}

@Preview(showBackground = true)
@Composable
fun ManagerAppScreenPreview() {
    UserSUTheme {
        ManagerAppContent()
    }
}
