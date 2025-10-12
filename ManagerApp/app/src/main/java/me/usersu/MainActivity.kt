package me.usersu

import android.net.Uri
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import me.usersu.ui.theme.UserSUTheme
import java.io.File
import androidx.activity.compose.rememberLauncherForActivityResult

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            UserSUTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    InstallUserSU()
                }
            }
        }
    }
}

@Composable
fun InstallUserSU(modifier: Modifier = Modifier) {
    val context = LocalContext.current
    var installStatus by remember { mutableStateOf("Ready to install UserSU") }

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

    Column(
        modifier = modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(text = installStatus)
        Button(
            onClick = {
                pickFileLauncher.launch(arrayOf("application/x-tar", "application/gzip", "application/x-xz"))
            }
        ) {
            Text("Select UserSU Tarball and Install")
        }
    }
}

@Preview(showBackground = true)
@Composable
fun InstallUserSUPreview() {
    UserSUTheme {
        InstallUserSU()
    }
}
