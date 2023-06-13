// JNI callback function
extern "C" JNIEXPORT void JNICALL
    dapiCoreGetStatus(JNIEnv* env,
                                                             jclass clazz,
                                                             jlong runtime_ptr,
                                                             jstring address,
                                                             jobject callback) {
    // Convert Java String to C string
    const char* address_str = env->GetStringUTFChars(address, nullptr);

    // Call rust
    dapi_core_get_status(runtime_ptr, address_str, [](void* ctx, GetStatusResponse response) {
        JNIEnv* env = reinterpret_cast<JNIEnv*>(ctx);

        // Find the GetStatusResponse class
        jclass responseClazz = env->FindClass("org/dashj/example/GetStatusResponse");
        jobject javaResponse = env->AllocObject(responseClazz);

        // Set fields for the Version structure
        jclass versionClass = env->FindClass("org/dashj/example/Version");
        jobject versionObject = env->AllocObject(versionClass);

        jfieldID protocolField = env->GetFieldID(versionClass, "protocol", "I");
        env->SetIntField(versionObject, protocolField, response.version->protocol);

        jfieldID softwareField = env->GetFieldID(versionClass, "software", "I");
        env->SetIntField(versionObject, softwareField, response.version->software);

        jfieldID agentField = env->GetFieldID(versionClass, "agent", "Ljava/lang/String;");
        jstring agentString = env->NewStringUTF(response.version->agent);
        env->SetObjectField(versionObject, agentField, agentString);

        // Set fields for the Time structure
        jclass timeClass = env->FindClass("org/dashj/example/Time");
        jobject timeObject = env->AllocObject(timeClass);

        jfieldID nowField = env->GetFieldID(timeClass, "now", "J");
        env->SetLongField(timeObject, nowField, response.time->now);

        jfieldID offsetField = env->GetFieldID(timeClass, "offset", "I");
        env->SetIntField(timeObject, offsetField, response.time->offset);

        jfieldID medianField = env->GetFieldID(timeClass, "median", "J");
        env->SetLongField(timeObject, medianField, response.time->median);

        // Attach the Version and Time objects to the response object
        jfieldID versionObjField = env->GetFieldID(responseClazz, "version", "Lorg/dashj/example/Version;");
        env->SetObjectField(javaResponse, versionObjField, versionObject);

        jfieldID timeObjField = env->GetFieldID(responseClazz, "time", "Lorg/dashj/example/Time;");
        env->SetObjectField(javaResponse, timeObjField, timeObject);

        // Call the Java callback with the response object
        jclass callbackInterface = env->GetObjectClass(callback);
        jmethodID onResponseMethod = env->GetMethodID(callbackInterface, "onResponse", "(Lorg/dashj/example/GetStatusResponse;)V");
        env->CallVoidMethod(callback, onResponseMethod, javaResponse);
    }, env);

    // Release the address string
    env->ReleaseStringUTFChars(address, address_str);
}