public class RustRunner {
    static {
        System.loadLibrary("DashSharedCore");
    }

    private native long createRuntime();
    private native void getStatus(long runtimePtr, RustCallback callback, Object context);

    public void run() {
        long runtimePtr = createRuntime();
        getStatus(runtimePtr, (context, result) -> {
            System.out.println("Async result: " + result);
        }, null);
        // Other code
    }

    public interface RustCallback {
        void invoke(Object context, int result);
    }
}