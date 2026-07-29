try { work(); } catch { }
try { work(); } catch (error) { /* ignored */ }
try { work(); } catch (error) { recover(error); }
