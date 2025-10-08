package me.usersu;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.util.Log;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.zip.GZIPInputStream;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;

public class TarballExtractActivity extends Activity {
    private static final String TAG = "UserSU";
    private static final int PICK_TAR_FILE = 1;
    private static final int PICK_UPDATE_FILE = 2;
    private TextView statusText;
    private TextView deviceInfoText;
    private Button selectButton;
    private Button updateButton;
    private Button uninstallButton;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Create scrollable layout
        ScrollView scrollView = new ScrollView(this);
        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        layout.setPadding(40, 40, 40, 40);
        
        // Device info section
        deviceInfoText = new TextView(this);
        deviceInfoText.setText(getDeviceInfo());
        deviceInfoText.setTextSize(12);
        deviceInfoText.setPadding(20, 20, 20, 20);
        deviceInfoText.setBackgroundColor(0xFFEEEEEE);
        
        // Status text
        statusText = new TextView(this);
        statusText.setText("UserSU Installer\n\n" +
                "Expected tarball structure:\n" +
                "  bin/su\n" +
                "  lib/libfakeroot.so\n\n" +
                "Tap 'Install' to choose a .tar.gz file.");
        statusText.setTextSize(14);
        statusText.setPadding(0, 40, 0, 40);
        
        // Install button
        selectButton = new Button(this);
        selectButton.setText("Install from Tarball");
        selectButton.setOnClickListener(v -> selectTarball(PICK_TAR_FILE));
        
        // Update button
        updateButton = new Button(this);
        updateButton.setText("Update from Tarball");
        updateButton.setOnClickListener(v -> selectTarball(PICK_UPDATE_FILE));
        updateButton.setEnabled(false);
        
        // Uninstall button
        uninstallButton = new Button(this);
        uninstallButton.setText("Uninstall");
        uninstallButton.setOnClickListener(v -> confirmUninstall());
        uninstallButton.setEnabled(false);
        
        Button openTerminalButton = new Button(this);
        openTerminalButton.setText("Open Terminal");
        openTerminalButton.setLayoutParams(new LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.WRAP_CONTENT,
            LinearLayout.LayoutParams.WRAP_CONTENT
        ));
        
        openTerminalButton.setOnClickListener(v -> {
            Intent intent = new Intent(MainActivity.this, TerminalActivity.class);
            startActivity(intent);
        });
        
        layout.addView(deviceInfoText);
        layout.addView(statusText);
        layout.addView(selectButton);
        layout.addView(updateButton);
        layout.addView(uninstallButton);
        layout.addView(openTerminalButton);
        
        scrollView.addView(layout);
        setContentView(scrollView);
        
        // Check installation status
        updateUIState();
    }
    
    private String getDeviceInfo() {
        StringBuilder info = new StringBuilder();
        info.append("═══ DEVICE INFORMATION ═══\n\n");
        info.append("Android Version: ").append(Build.VERSION.RELEASE)
            .append(" (API ").append(Build.VERSION.SDK_INT).append(")\n");
        info.append("Architecture: ").append(Build.SUPPORTED_ABIS[0]);
        if (Build.SUPPORTED_ABIS.length > 1) {
            info.append(", ").append(Build.SUPPORTED_ABIS[1]);
        }
        info.append("\n");
        info.append("Device: ").append(Build.DEVICE).append("\n");
        info.append("Model: ").append(Build.MODEL).append("\n");
        info.append("Manufacturer: ").append(Build.MANUFACTURER);
        return info.toString();
    }
    
    private void updateUIState() {
        File binDir = new File(getFilesDir(), "bin");
        File suBinary = new File(binDir, "su");
        boolean isInstalled = suBinary.exists();
        
        if (isInstalled) {
            statusText.setText("Files installed!\n\n" +
                    "Location: " + getFilesDir().getAbsolutePath() + "\n" +
                    "Binary: " + suBinary.getAbsolutePath() + "\n\n" +
                    "You can update or uninstall using the buttons below.");
            selectButton.setEnabled(false);
            updateButton.setEnabled(true);
            uninstallButton.setEnabled(true);
        } else {
            statusText.setText("UserSU Installer\n\n" +
                    "Expected tarball structure:\n" +
                    "  bin/su\n" +
                    "  lib/libfakeroot.so\n\n" +
                    "Tap 'Install from Tarball' to begin.");
            selectButton.setEnabled(true);
            updateButton.setEnabled(false);
            uninstallButton.setEnabled(false);
        }
    }
    
    private void selectTarball(int requestCode) {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        startActivityForResult(intent, requestCode);
    }
    
    private void confirmUninstall() {
        new AlertDialog.Builder(this)
            .setTitle("Uninstall UserSU")
            .setMessage("This will remove all extracted files including:\n• bin/su\n• lib/\n• rootfs/\n\nContinue?")
            .setPositiveButton("Uninstall", (dialog, which) -> uninstallFiles())
            .setNegativeButton("Cancel", null)
            .show();
    }
    
    private void uninstallFiles() {
        statusText.setText("Uninstalling...");
        
        new Thread(() -> {
            try {
                int deleted = 0;
                File filesDir = getFilesDir();
                
                // Delete bin directory
                File binDir = new File(filesDir, "bin");
                deleted += deleteDirectory(binDir);
                
                // Delete lib directory
                File libDir = new File(filesDir, "lib");
                deleted += deleteDirectory(libDir);
                
                // Delete rootfs directory
                File rootfsDir = new File(filesDir, "rootfs");
                deleted += deleteDirectory(rootfsDir);
                
                final int count = deleted;
                runOnUiThread(() -> {
                    statusText.setText("Uninstall complete!\n\n" +
                            "Removed " + count + " files.\n\n" +
                            "You can now install a new version.");
                    updateUIState();
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Uninstall failed", e);
                runOnUiThread(() -> {
                    statusText.setText("Uninstall failed:\n" + e.getMessage());
                    updateUIState();
                });
            }
        }).start();
    }
    
    private int deleteDirectory(File directory) {
        int count = 0;
        if (directory.exists()) {
            File[] files = directory.listFiles();
            if (files != null) {
                for (File file : files) {
                    if (file.isDirectory()) {
                        count += deleteDirectory(file);
                    } else {
                        if (file.delete()) {
                            count++;
                            Log.i(TAG, "Deleted: " + file.getAbsolutePath());
                        }
                    }
                }
            }
            if (directory.delete()) {
                Log.i(TAG, "Deleted directory: " + directory.getAbsolutePath());
            }
        }
        return count;
    }
    
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        
        if ((requestCode == PICK_TAR_FILE || requestCode == PICK_UPDATE_FILE) 
                && resultCode == RESULT_OK) {
            if (data != null) {
                Uri uri = data.getData();
                boolean isUpdate = (requestCode == PICK_UPDATE_FILE);
                extractTarball(uri, isUpdate);
            }
        }
    }
    
    private void extractTarball(Uri uri, boolean isUpdate) {
        String action = isUpdate ? "Updating" : "Extracting";
        statusText.setText(action + " tarball...");
        
        new Thread(() -> {
            try {
                InputStream inputStream = getContentResolver().openInputStream(uri);
                if (inputStream == null) {
                    throw new IOException("Cannot open tarball");
                }
                
                // Handle .tar.gz
                InputStream decompressed = inputStream;
                if (uri.toString().endsWith(".gz")) {
                    decompressed = new GZIPInputStream(inputStream);
                }
                
                TarArchiveInputStream tarInput = new TarArchiveInputStream(decompressed);
                
                File filesDir = getFilesDir();
                File rootfsDir = new File(filesDir, "rootfs");
                rootfsDir.mkdirs();
                
                int extracted = 0;
                TarArchiveEntry entry;
                
                while ((entry = (TarArchiveEntry) tarInput.getNextTarEntry()) != null) {
                    String name = entry.getName();
                    
                    // Skip directories
                    if (entry.isDirectory()) {
                        continue;
                    }
                    
                    // Determine output location
                    File outFile;
                    if (name.startsWith("bin/")) {
                        File binDir = new File(filesDir, "bin");
                        binDir.mkdirs();
                        outFile = new File(binDir, new File(name).getName());
                    } else if (name.startsWith("lib/")) {
                        File libDir = new File(filesDir, "lib");
                        libDir.mkdirs();
                        outFile = new File(libDir, new File(name).getName());
                    } else if (name.startsWith("fs/")) {
                        outFile = new File(rootfsDir, name.substring(3));
                        outFile.getParentFile().mkdirs();
                    } else {
                        // Skip unknown files
                        continue;
                    }
                    
                    // Extract file
                    FileOutputStream out = new FileOutputStream(outFile);
                    byte[] buffer = new byte[8192];
                    int read;
                    while ((read = tarInput.read(buffer)) != -1) {
                        out.write(buffer, 0, read);
                    }
                    out.close();
                    
                    // Make executable if in bin/
                    if (name.startsWith("bin/")) {
                        outFile.setExecutable(true, false);
                    }
                    
                    extracted++;
                    Log.i(TAG, (isUpdate ? "Updated: " : "Extracted: ") + outFile.getAbsolutePath());
                }
                
                tarInput.close();
                
                final int count = extracted;
                runOnUiThread(() -> {
                    statusText.setText((isUpdate ? "Update" : "Installation") + " complete!\n\n" +
                            "Extracted " + count + " files to:\n" +
                            filesDir.getAbsolutePath() + "\n\n" +
                            "You can now use the su binary.");
                    updateUIState();
                });
                
            } catch (Exception e) {
                Log.e(TAG, (isUpdate ? "Update" : "Extraction") + " failed", e);
                runOnUiThread(() -> {
                    statusText.setText((isUpdate ? "Update" : "Installation") + " failed:\n" + 
                            e.getMessage() + "\n\n" +
                            "Make sure the tarball has the correct structure:\n" +
                            "  bin/su\n" +
                            "  lib/libfakeroot.so");
                    updateUIState();
                });
            }
        }).start();
    }
}
