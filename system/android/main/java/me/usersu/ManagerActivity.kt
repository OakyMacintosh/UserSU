package me.usersu.manager

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Terminal
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import com.google.android.material.color.DynamicColors
import java.io.BufferedReader
import java.io.InputStreamReader

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        DynamicColors.applyToActivitiesIfAvailable(application)
        super.onCreate(savedInstanceState)

        setContent {
            val ctx = LocalContext.current
            val colorScheme =
                if (isSystemInDarkTheme()) dynamicDarkColorScheme(ctx)
                else dynamicLightColorScheme(ctx)

            MaterialTheme(colorScheme = colorScheme) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    UserSUManagerUI()
                }
            }
        }
    }
}

@Composable
fun UserSUManagerUI() {
    var logs by remember { mutableStateOf(listOf<String>()) }
    var command by remember { mutableStateOf("") }

    Scaffold(
        topBar = {
            SmallTopAppBar(
                title = { Text("UserSU Manager") },
                navigationIcon = {
                    Icon(
                        imageVector = Icons.Default.Terminal,
                        contentDescription = "terminal"
                    )
                }
            );
            Button(
                onClick = {
                    val ctx = LocalContext.current
                    ctx.startActivity(Intent(ctx, TarballExtractActivity::class.java))
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Install / Manage Tarball")
            }
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                text = { Text("Run") },
                onClick = {
                    if (command.isNotBlank()) {
                        val output = runShell(command)
                        logs = logs + "$ $command" + output
                        command = ""
                    }
                }
            )
        }
    ) { padding ->
        Column(
            Modifier
                .padding(padding)
                .fillMaxSize()
                .padding(12.dp)
        ) {
            OutlinedTextField(
                value = command,
                onValueChange = { command = it },
                label = { Text("Command") },
                modifier = Modifier.fillMaxWidth()
            )

            Spacer(Modifier.height(12.dp))

            Text(
                "Output:",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.secondary
            )

            LazyColumn(
                modifier = Modifier
                    .weight(1f)
                    .padding(top = 8.dp)
            ) {
                items(logs) { line ->
                    Text(
                        text = line,
                        fontFamily = FontFamily.Monospace,
                        textAlign = TextAlign.Start,
                        color = MaterialTheme.colorScheme.onBackground
                    )
                }
            }
        }
    }
}

/**
 * Executes a shell command (non-root for demo).
 * In your real implementation, you'd wrap this with UserSU binary logic.
 */
fun runShell(cmd: String): List<String> {
    return try {
        val process = Runtime.getRuntime().exec(cmd)
        val reader = BufferedReader(InputStreamReader(process.inputStream))
        val output = reader.readLines()
        reader.close()
        output
    } catch (e: Exception) {
        listOf("Error: ${e.message}")
    }
}
