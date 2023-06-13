import java.nio.file.Path;
import java.nio.file.Paths;

public class example {
    // TODO: make a build for android archs and targets
    static {
        Path p = Paths.get("DashSharedCore.dylib");
        System.load(p.toAbsolutePath().toString());
    }

    private native long create_runtime();
    private native void destroy_runtime(long runtimePtr);
    private native void dapi_core_get_status(long runtimePtr, String address, GetStatusResponseCallback callback);

    public void run() {
        long runtimePtr = create_runtime();
        dapi_core_get_status(runtimePtr, "0.0.0.0:0", (result) -> {
            System.out.println("Async result: " + result);
        });
        destroy_runtime(runtimePtr);
    }


    public interface GetStatusResponseCallback {
        void invoke(GetStatusResponse result);
    }
}
