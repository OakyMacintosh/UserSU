package me.usersu

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.google.android.material.color.DynamicColors
import kotlinx.coroutines.*
import java.io.*
import java.util.zip.GZIPInputStream
import org.apache.commons.compress.archivers.tar.*

class TarballExtractActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        DynamicColors.applyToActivitiesIfAvailable(application)
        super.onCreate(savedInstanceState)

        setContent {
            val ctx = LocalContext.current
            val scheme =
                if (isSystemInDarkTheme()) dynamicDarkColorScheme(ctx)
                else dynamicLightColorScheme(ctx)

            MaterialTheme(colorScheme = scheme) {
                Surface(Modifier.fillMaxSize()) {
                    TarballExtractScreen()
                }
            }
        }
    }
}

@Composable
fun TarballExtractScreen() {
    val ctx = LocalContext.current
    var status by remember { mutableStateOf("Ready to install a tarball.") }
    val scope = rememberCoroutineScope()

    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
        uri ?: return@rememberLauncherForActivityResult
        scope.launch {
            status = extractTarball(ctx.filesDir, uri)
        }
    }

    Scaffold(
        topBar = { SmallTopAppBar(title = { Text("UserSU Installer") }) }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .padding(16.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(status)
            Button(
                onClick = { launcher.launch(arrayOf("*/*")) },
                modifier = Modifier.fillMaxWidth()
            ) { Text("Install from Tarball") }

            Button(
                onClick = {
                    File(ctx.filesDir, "bin").deleteRecursively()
                    File(ctx.filesDir, "lib").deleteRecursively()
                    File(ctx.filesDir, "rootfs").deleteRecursively()
                    status = "Uninstall complete."
                },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)
            ) { Text("Uninstall") }
        }
    }
}

suspend fun extractTarball(filesDir: File, uri: Uri): String = withContext(Dispatchers.IO) {
    val context = androidx.compose.ui.platform.LocalContext.current
    return@withContext try {
        val inputStream = context.contentResolver.openInputStream(uri)
            ?: return@withContext "Cannot open tarball."
        val decompressed = if (uri.toString().endsWith(".gz")) GZIPInputStream(inputStream) else inputStream
        val tarInput = TarArchiveInputStream(decompressed)

        var count = 0
        var entry: TarArchiveEntry?

        while (true) {
            entry = tarInput.nextEntry as? TarArchiveEntry ?: break
            if (entry.isDirectory) continue

            val name = entry.name
            val outFile = when {
                name.startsWith("bin/") -> File(filesDir, name)
                name.startsWith("lib/") -> File(filesDir, name)
                name.startsWith("fs/") -> File(filesDir, "rootfs/${name.removePrefix("fs/")}")
                else -> continue
            }

            outFile.parentFile?.mkdirs()
            FileOutputStream(outFile).use { tarInput.copyTo(it) }
            if (name.startsWith("bin/")) outFile.setExecutable(true)
            count++
        }

        tarInput.close()
        "Extracted $count files successfully."
    } catch (e: Exception) {
        "Extraction failed: ${e.message}"
    }
}
