#import <AVFoundation/AVFoundation.h>
// Request only after the user presses Record. macOS owns the permission dialog.
int mypdf_microphone_permission(int request) {
    AVAuthorizationStatus status = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
    if (request && status == AVAuthorizationStatusNotDetermined) {
        [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted) { (void)granted; }];
    }
    return (int)status;
}
