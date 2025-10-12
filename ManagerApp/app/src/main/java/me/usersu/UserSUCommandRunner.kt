package me.usersu

import android.content.Context
import java.io.File
import java.io.BufferedReader
import java.io.InputStreamReader

object UserSUCommandRunner {

    fun executeUserSUCommand(context: Context, command: String): String {
        val usudPath = File(context.filesDir, "usersu_binaries/usud")
        if (!usudPath.exists() || !usudPath.canExecute()) {
            return "Error: usud binary not found or not executable at ${usudPath.absolutePath}"
        }

        return try {
            val processBuilder = ProcessBuilder(usudPath.absolutePath, command)
            processBuilder.redirectErrorStream(true)
            val process = processBuilder.start()

            val reader = BufferedReader(InputStreamReader(process.inputStream))
            val output = StringBuilder()
            var line: String?
            while (reader.readLine().also { line = it } != null) {
                output.append(line).append("\n")
            }
            process.waitFor()
            output.toString().trim()
        } catch (e: Exception) {
            e.printStackTrace()
            "Error executing command: ${e.message}"
        }
    }
}
