package dev.usersu.manager

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.*
import androidx.compose.foundation.Image
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.layout.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import android.content.res.AssetManager
import android.graphics.BitmapFactory
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.material3.Text
import androidx.compose.material3.MaterialTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            UserSUManagerTheme {
                Surface(color = MaterialTheme.colorScheme.background) {
                    UserSUNavApp()
                }
            }
        }
    }
}

@Composable
fun UserSUNavApp() {
    val navController = rememberNavController()
    val items = listOf(
        NavItem("Home", Icons.Default.Home, "home"),
        NavItem("Users", Icons.Default.Person, "users"),
        NavItem("Settings", Icons.Default.Settings, "settings")
    )
    var selectedItem by remember { mutableStateOf(0) }

    Scaffold(
        bottomBar = {
            NavigationBar {
                items.forEachIndexed { index, item ->
                    NavigationBarItem(
                        icon = { Icon(item.icon, contentDescription = item.label) },
                        label = { Text(item.label) },
                        selected = selectedItem == index,
                        onClick = {
                            selectedItem = index
                            navController.navigate(item.route) {
                                popUpTo(navController.graph.startDestinationId) { saveState = true }
                                launchSingleTop = true
                                restoreState = true
                            }
                        }
                    )
                }
            }
        }
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = "home",
            modifier = Modifier.padding(innerPadding)
        ) {
            composable("home") { HomeScreen() }
            composable("users") { UsersScreen() }
            composable("settings") { SettingsScreen() }
        }
    }
}

data class NavItem(val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector, val route: String)

@Composable
fun rememberImageBitmap(assets: AssetManager, fileName: String): ImageBitmap {
    return remember {
        assets.open(fileName).use { stream ->
            BitmapFactory.decodeStream(stream)!!.asImageBitmap()
        }
    }
}


@Composable
fun HomeScreen() {
    val context = LocalContext.current
    Box(
        modifier = Modifier
            .fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Show the logo from assets
            Image(
                painter = rememberImageBitmap(context.assets, "logo.png"),
                contentDescription = "UserSU Logo",
                modifier = Modifier.size(96.dp)
            )
            Spacer(modifier = Modifier.height(24.dp))
            Text("Welcome to UserSU Manager", style = MaterialTheme.typography.headlineMedium)
        }
    }
}


@Composable
fun UsersScreen() {
    CenteredText("Manage Users Here")
}

@Composable
fun SettingsScreen() {
    CenteredText("Settings")
}

@Composable
fun CenteredText(text: String) {
    androidx.compose.foundation.layout.Box(
        modifier = Modifier
            .fillMaxSize(),
        contentAlignment = androidx.compose.ui.Alignment.Center
    ) {
        Text(text, style = MaterialTheme.typography.headlineMedium)
    }
}