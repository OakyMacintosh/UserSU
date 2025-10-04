package me.usersu;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
import android.widget.Button;
import android.widget.LinearLayout;
import android.view.ViewGroup;
import android.util.Log;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;

public class SimpleExtractActivity extends Activity {
    private static final String TAG = "UserSU";
    private TextView statusText;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Create simple UI
        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        layout.setPadding(40, 40, 40, 40);
        
        statusText = new TextView(this);
        statusText.setText("UserSU Installer\n\nTap 'Extract Files' to install native binaries.");
        statusText.setTextSize(16);
        statusText.setPadding(0, 0, 0, 40);
        
        Button extractButton = new Button(this);
        extractButton.setText("Extract Files");
        extractButton.setOnClickListener(v -> extractFiles());
        
        layout.addView(statusText);
        layout.addView(extractButton);
        
        setContentView(layout);
        
        // Check if already extracted
        File binDir = new File(getFilesDir(), "bin");
        if (binDir.exists()) {
            statusText.setText("Files already extracted!\n\nLocation: " + getFilesDir().getAbsolutePath());
        }
    }
    
    private void extractFiles() {
        statusText.setText("Extracting files...");
        
        new Thread(() -> {
            try {
                File filesDir = getFilesDir();
                File binDir = new File(filesDir, "bin");
                File libDir = new File(filesDir, "lib");
                File rootfsDir = new File(filesDir, "rootfs");
                
                binDir.mkdirs();
                libDir.mkdirs();
                rootfsDir.mkdirs();
                
                // Get device ABI
                String abi = android.os.Build.CPU_ABI;
                String assetDir = abi.contains("arm64") || abi.contains("aarch64") ? "arm64-v8a" : "armeabi-v7a";
                
                Log.i(TAG, "Extracting for ABI: " + abi + " from assets/" + assetDir);
                
                // Extract from assets
                extractAssetDir(assetDir + "/bin", binDir);
                extractAssetDir(assetDir + "/lib", libDir);
                
                runOnUiThread(() -> {
                    statusText.setText("✓ Extraction complete!\n\n" +
                            "Files location:\n" + filesDir.getAbsolutePath() + "\n\n" +
                            "Binaries in: bin/\n" +
                            "Libraries in: lib/\n" +
                            "Root filesystem: rootfs/");
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Extraction failed", e);
                runOnUiThread(() -> {
                    statusText.setText("✗ Extraction failed:\n" + e.getMessage());
                });
            }
        }).start();
    }
    
    private void extractAssetDir(String assetPath, File targetDir) throws Exception {
        String[] files = getAssets().list(assetPath);
        
        if (files == null || files.length == 0) {
            return;
        }
        
        for (String filename : files) {
            if (filename.equals("placeholder")) continue;
            
            String fullAssetPath = assetPath + "/" + filename;
            File outFile = new File(targetDir, filename);
            
            InputStream in = getAssets().open(fullAssetPath);
            FileOutputStream out = new FileOutputStream(outFile);
            
            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
            
            in.close();
            out.close();
            
            // Make executable if in bin/
            if (assetPath.contains("/bin")) {
                outFile.setExecutable(true, false);
            }
            
            Log.i(TAG, "Extracted: " + outFile.getName());
        }
    }
}