package com.example.terminal;

import android.app.Activity;
import android.os.Bundle;
import android.view.ViewGroup;
import android.widget.FrameLayout;

import com.termux.terminal.TerminalSession;
import com.termux.terminal.TerminalSessionClient;
import com.termux.terminal.TerminalEmulator;
import com.termux.view.TerminalView;
import com.termux.view.TerminalViewClient;

public class TerminalActivity extends Activity {
    
    private TerminalView terminalView;
    private TerminalSession terminalSession;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Create FrameLayout as root view
        FrameLayout rootLayout = new FrameLayout(this);
        rootLayout.setLayoutParams(new ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.MATCH_PARENT
        ));
        
        // Create TerminalView
        terminalView = new TerminalView(this, null);
        terminalView.setLayoutParams(new FrameLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.MATCH_PARENT
        ));
        
        // Set TerminalViewClient
        terminalView.setTerminalViewClient(new TerminalViewClient() {
            @Override
            public float onScale(float scale) {
                return scale; // Handle pinch zoom
            }
            
            @Override
            public void onSingleTapUp(android.view.MotionEvent e) {
                // Handle tap events
            }
            
            @Override
            public boolean shouldBackButtonBeMappedToEscape() {
                return false;
            }
            
            @Override
            public void copyModeChanged(boolean copyMode) {
                // Handle copy mode changes
            }
            
            @Override
            public boolean onKeyDown(int keyCode, android.view.KeyEvent e, TerminalSession session) {
                return false;
            }
            
            @Override
            public boolean onKeyUp(int keyCode, android.view.KeyEvent e) {
                return false;
            }
            
            @Override
            public boolean readControlKey() {
                return false;
            }
            
            @Override
            public boolean readAltKey() {
                return false;
            }
            
            @Override
            public boolean onCodePoint(int codePoint, boolean ctrlDown, TerminalSession session) {
                return false;
            }
            
            @Override
            public boolean onLongPress(android.view.MotionEvent event) {
                return false;
            }
        });
        
        // Create terminal session
        createTerminalSession();
        
        // Add view to layout
        rootLayout.addView(terminalView);
        setContentView(rootLayout);
    }
    
    private void createTerminalSession() {
        // Session configuration
        String shellPath = "/system/bin/sh";
        String workingDirectory = getFilesDir().getAbsolutePath();
        String[] environment = buildEnvironment();
        String[] arguments = new String[]{};
        
        // Create session with callback
        terminalSession = new TerminalSession(
            shellPath,
            workingDirectory,
            arguments,
            environment,
            new TerminalSessionClient() {
                @Override
                public void onTextChanged(TerminalSession changedSession) {
                    terminalView.onScreenUpdated();
                }
                
                @Override
                public void onTitleChanged(TerminalSession changedSession) {
                    // Update activity title if needed
                    setTitle(changedSession.getTitle());
                }
                
                @Override
                public void onSessionFinished(TerminalSession finishedSession) {
                    // Handle session end
                    finish();
                }
                
                @Override
                public void onCopyTextToClipboard(TerminalSession session, String text) {
                    android.content.ClipboardManager clipboard = 
                        (android.content.ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
                    android.content.ClipData clip = android.content.ClipData.newPlainText("Terminal", text);
                    clipboard.setPrimaryClip(clip);
                }
                
                @Override
                public void onPasteTextFromClipboard(TerminalSession session) {
                    android.content.ClipboardManager clipboard = 
                        (android.content.ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
                    if (clipboard.hasPrimaryClip()) {
                        android.content.ClipData clip = clipboard.getPrimaryClip();
                        if (clip != null && clip.getItemCount() > 0) {
                            CharSequence text = clip.getItemAt(0).getText();
                            if (text != null) {
                                session.write(text.toString());
                            }
                        }
                    }
                }
                
                @Override
                public void onBell(TerminalSession session) {
                    // Handle bell/beep
                }
                
                @Override
                public void onColorsChanged(TerminalSession session) {
                    terminalView.onScreenUpdated();
                }
                
                @Override
                public void onTerminalCursorStateChange(boolean state) {
                    // Handle cursor state
                }
                
                @Override
                public Integer getTerminalCursorStyle() {
                    return TerminalEmulator.TERMINAL_CURSOR_STYLE_BLOCK;
                }
                
                @Override
                public void setTerminalShellPid(TerminalSession session, int pid) {
                    // Store shell PID if needed
                }
                
                @Override
                public void onTerminalEmulatorSet() {
                    // Terminal emulator ready
                }
            }
        );
        
        // Attach session to view
        terminalView.attachSession(terminalSession);
    }
    
    private String[] buildEnvironment() {
        String[] env = new String[]{
            "TERM=xterm-256color",
            "HOME=" + getFilesDir().getAbsolutePath(),
            "PATH=/system/bin:/system/xbin",
            "TMPDIR=" + getCacheDir().getAbsolutePath()
        };
        return env;
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (terminalSession != null) {
            terminalSession.finishIfRunning();
        }
    }
    
    @Override
    public void onBackPressed() {
        // Send Escape key or exit
        if (terminalSession != null && terminalSession.isRunning()) {
            terminalSession.write("\u001b"); // ESC key
        } else {
            super.onBackPressed();
        }
    }
}
